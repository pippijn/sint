use sint_core::logic::GameLogic;
use sint_core::types::{Action, GamePhase, GameState, HazardType, PlayerId};
use std::fmt::Write;

pub fn format_trajectory(initial_state: GameState, path: Vec<(PlayerId, Action)>) -> Vec<String> {
    let mut state = initial_state;
    let mut current_round = state.turn_count;
    let mut round_start_hull = state.hull_integrity;
    let mut round_start_hazards = count_hazards(&state);

    let mut rounds_output = Vec::new();
    let mut current_buffer = String::new();

    writeln!(current_buffer, "\n=== BEST TRAJECTORY FOUND ===").unwrap();
    writeln!(
        current_buffer,
        "Start: Hull {}, Boss {}, Players {}",
        state.hull_integrity,
        state.enemy.name,
        state.players.len()
    )
    .unwrap();

    for (pid, action) in path {
        let prev_phase = state.phase;

        // Apply
        let res = GameLogic::apply_action(state.clone(), &pid, action.clone(), None);
        match res {
            Ok(new_state) => {
                match &action {
                    Action::Chat { message } => {
                        if message.starts_with("[MACRO]") {
                            writeln!(
                                current_buffer,
                                "\n  {} plans: {}",
                                pid,
                                message.replace("[MACRO] ", "")
                            )
                            .unwrap();
                        }
                    }
                    Action::VoteReady { .. } => {} // Silent
                    Action::Pass => {
                        writeln!(current_buffer, "  {} passes.", pid).unwrap();
                    }
                    _ => {
                        writeln!(current_buffer, "    -> {:?} ({})", action, pid).unwrap();
                    }
                }

                state = new_state;

                if state.phase != prev_phase {
                    if state.turn_count > current_round {
                        // Finish previous round
                        let results = format_results(
                            round_start_hull,
                            state.hull_integrity,
                            round_start_hazards,
                            count_hazards(&state),
                        );
                        write!(current_buffer, "{}", results).unwrap();

                        rounds_output.push(current_buffer);
                        current_buffer = String::new();

                        current_round = state.turn_count;
                        round_start_hull = state.hull_integrity;
                        round_start_hazards = count_hazards(&state);

                        writeln!(current_buffer, "\n--- ROUND {} ---", current_round).unwrap();
                    }

                    if state.phase == GamePhase::MorningReport
                        && prev_phase != GamePhase::MorningReport
                    {
                        if let Some(card) = &state.latest_event {
                            writeln!(
                                current_buffer,
                                "[EVENT] {} - {}",
                                card.title, card.description
                            )
                            .unwrap();
                        }
                        if !state.active_situations.is_empty() {
                            let names: Vec<String> = state
                                .active_situations
                                .iter()
                                .map(|c| c.title.clone())
                                .collect();
                            writeln!(current_buffer, "[ACTIVE] {}", names.join(", ")).unwrap();
                        }
                    } else if state.phase == GamePhase::TacticalPlanning
                        && prev_phase != GamePhase::TacticalPlanning
                    {
                        write!(current_buffer, "{}", format_planning_context(&state)).unwrap();
                    }
                }
            }
            Err(e) => {
                writeln!(
                    current_buffer,
                    "  ERROR REPLAYING: {} failed to {:?}: {}",
                    pid, action, e
                )
                .unwrap();
            }
        }
    }

    writeln!(current_buffer, "\n=== GAME OVER: {:?} ===", state.phase).unwrap();
    rounds_output.push(current_buffer);

    rounds_output
}

pub fn print_trajectory(initial_state: GameState, path: Vec<(PlayerId, Action)>) {
    let rounds = format_trajectory(initial_state, path);
    for r in rounds {
        print!("{}", r);
    }
}

fn format_planning_context(state: &GameState) -> String {
    let mut out = String::new();

    // 1. Enemy Intent
    if let Some(attack) = &state.enemy.next_attack {
        writeln!(
            out,
            "[ENEMY] {} targets Room {} with {:?}",
            state.enemy.name, attack.target_room, attack.effect
        )
        .unwrap();
    }

    // 2. Hazards
    let mut fire_rooms = Vec::new();
    let mut water_rooms = Vec::new();

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
        writeln!(out, "[HAZARDS] Fire: {}", fire_rooms.join(", ")).unwrap();
    }
    if !water_rooms.is_empty() {
        writeln!(out, "[HAZARDS] Water: {}", water_rooms.join(", ")).unwrap();
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
    writeln!(out, "[PLAYERS] {}", players_info.join(" | ")).unwrap();

    // 5. Status
    writeln!(
        out,
        "[STATUS] Hull: {} | Boss HP: {} | Total Hazards: {}",
        state.hull_integrity,
        state.enemy.hp,
        count_hazards(state)
    )
    .unwrap();
    writeln!(out, "[TEAM] Actions:").unwrap();

    out
}

fn format_results(old_hull: i32, new_hull: i32, old_hazards: usize, new_hazards: usize) -> String {
    let mut out = String::new();
    writeln!(out, "[RESULTS]").unwrap();
    if new_hull < old_hull {
        writeln!(out, "  ! Hull Damage: {} -> {}", old_hull, new_hull).unwrap();
    } else {
        writeln!(out, "  - Hull Stable ({})", new_hull).unwrap();
    }

    if new_hazards > old_hazards {
        writeln!(
            out,
            "  ! Hazards Increased: {} -> {}",
            old_hazards, new_hazards
        )
        .unwrap();
    } else if new_hazards < old_hazards {
        writeln!(
            out,
            "  + Hazards Controlled: {} -> {}",
            old_hazards, new_hazards
        )
        .unwrap();
    }
    out
}

fn count_hazards(state: &GameState) -> usize {
    state.map.rooms.values().map(|r| r.hazards.len()).sum()
}
