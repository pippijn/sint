use sint_core::logic::GameLogic;
use sint_core::types::{Action, GamePhase, GameState, HazardType, PlayerId};

pub fn print_trajectory(initial_state: GameState, path: Vec<(PlayerId, Action)>) {
    let mut state = initial_state;
    let mut current_round = state.turn_count;
    let mut round_start_hull = state.hull_integrity;
    let mut round_start_hazards = count_hazards(&state);

    println!("\n=== BEST TRAJECTORY FOUND ===\n");
    println!(
        "Start: Hull {}, Boss {}, Players {}",
        state.hull_integrity,
        state.enemy.name,
        state.players.len()
    );

    // Group actions by "Batch" (between phase changes or round changes)
    // Actually, simply iterating and detecting Phase changes is easier.

    // We want to print context *before* the actions of a round (Event, Telegraph).
    // But the path only contains Player Actions.
    // Events happen *between* player actions (during phase transitions).

    // Initial Context (Round 1)
    // We start in Lobby usually, so we won't print context until Planning.
    // But verify runs `new_game` which starts in Lobby.
    // So loop handles it.

    for (pid, action) in path {
        let prev_phase = state.phase;

        // Apply
        let res = GameLogic::apply_action(state.clone(), &pid, action.clone(), None);
        match res {
            Ok(new_state) => {
                match &action {
                    Action::Chat { message } => {
                        if message.starts_with("[MACRO]") {
                            println!("\n  {} plans: {}", pid, message.replace("[MACRO] ", ""));
                        }
                    }
                    Action::VoteReady { .. } => {} // Silent
                    Action::Pass => {
                        println!("  {} passes.", pid);
                    }
                    _ => {
                        println!("    -> {:?} ({})", action, pid);
                    }
                }

                state = new_state;

                if state.phase != prev_phase {
                    if state.turn_count > current_round {
                        print_results(
                            round_start_hull,
                            state.hull_integrity,
                            round_start_hazards,
                            count_hazards(&state),
                        );
                        current_round = state.turn_count;
                        round_start_hull = state.hull_integrity;
                        round_start_hazards = count_hazards(&state);

                        println!("\n--- ROUND {} ---", current_round);
                        // Don't print header here, wait for phases
                    }

                    if state.phase == GamePhase::MorningReport
                        && prev_phase != GamePhase::MorningReport
                    {
                        // New Event usually appears here
                        if let Some(card) = &state.latest_event {
                            println!("[EVENT] {} - {}", card.title, card.description);
                        }
                        if !state.active_situations.is_empty() {
                            let names: Vec<String> = state
                                .active_situations
                                .iter()
                                .map(|c| c.title.clone())
                                .collect();
                            println!("[ACTIVE] {}", names.join(", "));
                        }
                    } else if state.phase == GamePhase::TacticalPlanning
                        && prev_phase != GamePhase::TacticalPlanning
                    {
                        // We just entered Planning. Telegraph should be ready.
                        print_planning_context(&state);
                    }
                }
            }
            Err(e) => {
                println!("  ERROR REPLAYING: {} failed to {:?}: {}", pid, action, e);
            }
        }
    }

    println!("\n=== GAME OVER: {:?} ===", state.phase);
}

fn print_planning_context(state: &GameState) {
    // 1. Enemy Intent
    if let Some(attack) = &state.enemy.next_attack {
        println!(
            "[ENEMY] {} targets Room {} with {:?}",
            state.enemy.name, attack.target_room, attack.effect
        );
    }

    // 2. Hazards
    let mut fire_rooms = Vec::new();
    let mut water_rooms = Vec::new();

    // Sort Room IDs for stability
    let mut room_ids: Vec<u32> = state.map.rooms.keys().cloned().collect();
    room_ids.sort();

    for rid in room_ids {
        if let Some(room) = state.map.rooms.get(&rid) {
            let fires = room
                .hazards
                .iter()
                .filter(|&h| *h == HazardType::Fire)
                .count();
            let waters = room
                .hazards
                .iter()
                .filter(|&h| *h == HazardType::Water)
                .count();

            if fires > 0 {
                fire_rooms.push(format!("{} (x{})", rid, fires));
            }
            if waters > 0 {
                water_rooms.push(format!("{} (x{})", rid, waters));
            }
        }
    }

    if !fire_rooms.is_empty() {
        println!("[HAZARDS] Fire: {}", fire_rooms.join(", "));
    }
    if !water_rooms.is_empty() {
        println!("[HAZARDS] Water: {}", water_rooms.join(", "));
    }

    // 3. Players
    let mut players_info = Vec::new();
    let mut pids: Vec<String> = state.players.keys().cloned().collect();
    pids.sort();
    for pid in pids {
        if let Some(p) = state.players.get(&pid) {
            let inv_str = if p.inventory.is_empty() {
                "".to_string()
            } else {
                format!(" {:?}", p.inventory)
            };
            players_info.push(format!("{}: R{}{}", pid, p.room_id, inv_str));
        }
    }
    println!("[PLAYERS] {}", players_info.join(" | "));

    // 5. Status
    println!(
        "[STATUS] Hull: {} | Boss HP: {} | Total Hazards: {}",
        state.hull_integrity,
        state.enemy.hp,
        count_hazards(state)
    );
    println!("[TEAM] Actions:");
}

fn print_results(old_hull: i32, new_hull: i32, old_hazards: usize, new_hazards: usize) {
    println!("[RESULTS]");
    if new_hull < old_hull {
        println!("  ! Hull Damage: {} -> {}", old_hull, new_hull);
    } else {
        println!("  - Hull Stable ({})", new_hull);
    }

    if new_hazards > old_hazards {
        println!("  ! Hazards Increased: {} -> {}", old_hazards, new_hazards);
    } else if new_hazards < old_hazards {
        println!("  + Hazards Controlled: {} -> {}", old_hazards, new_hazards);
    }
}

fn count_hazards(state: &GameState) -> usize {
    state.map.rooms.values().map(|r| r.hazards.len()).sum()
}
