use crate::driver::GameDriver;
use sint_core::types::{GameAction, GamePhase, GameState, HazardType, PlayerId};
use std::fmt::Write;

pub fn format_trajectory(
    initial_state: GameState,
    path: Vec<(PlayerId, GameAction)>,
) -> Vec<String> {
    // Replay now uses the Driver to implicitly handle VoteReady
    let mut driver = GameDriver::new(initial_state);

    // Tracking vars
    let mut current_round = driver.state.turn_count;
    let mut round_start_hull = driver.state.hull_integrity;
    let mut round_start_hazards = count_hazards(&driver.state);
    let mut last_enemy_name = driver.state.enemy.name.clone();

    let mut rounds_output = Vec::new();
    let mut current_buffer = String::new();

    writeln!(current_buffer, "\n=== BEST TRAJECTORY FOUND ===").unwrap();
    writeln!(
        current_buffer,
        "Start: Hull {}, Boss {}, Playersィ {}",
        driver.state.hull_integrity,
        last_enemy_name,
        driver.state.players.len()
    )
    .unwrap();

    // Initial context print (Round 1)
    if driver.state.phase == GamePhase::TacticalPlanning {
        write!(
            current_buffer,
            "\n--- ROUND {} ---\n{}",
            current_round,
            format_planning_context(&driver.state)
        )
        .unwrap();
    }

    for (pid, action) in path {
        let prev_phase = driver.state.phase;

        // Log action attempt
        match &action {
            GameAction::Chat { message } => {
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
            GameAction::VoteReady { .. } => {} // Should not exist in new path, but silent if so
            GameAction::Pass => {
                writeln!(current_buffer, "  {} passes.", pid).unwrap();
            }
            _ => {
                writeln!(current_buffer, "    -> {:?} ({})", action, pid).unwrap();
            }
        }

        // Apply via Driver
        match driver.apply(&pid, action.clone()) {
            Ok(_) => {
                // Detect Boss Defeat/Change
                if driver.state.enemy.name != last_enemy_name {
                    writeln!(
                        current_buffer,
                        "\n**************************************************"
                    )
                    .unwrap();
                    writeln!(current_buffer, "⚔️  BOSS DEFEATED: {}  ⚔️", last_enemy_name).unwrap();
                    writeln!(
                        current_buffer,
                        "💀  NEW CHALLENGER: {}  💀",
                        driver.state.enemy.name
                    )
                    .unwrap();
                    writeln!(
                        current_buffer,
                        "**************************************************\n"
                    )
                    .unwrap();
                    last_enemy_name = driver.state.enemy.name.clone();
                }

                // If phase changed or round incremented
                if driver.state.turn_count > current_round {
                    // Round Finished
                    let results = format_results(
                        round_start_hull,
                        driver.state.hull_integrity,
                        round_start_hazards,
                        count_hazards(&driver.state),
                    );
                    write!(current_buffer, "{}", results).unwrap();

                    rounds_output.push(current_buffer);
                    current_buffer = String::new();

                    current_round = driver.state.turn_count;
                    round_start_hull = driver.state.hull_integrity;
                    round_start_hazards = count_hazards(&driver.state);

                    writeln!(current_buffer, "\n--- ROUND {} ---", current_round).unwrap();

                    // If we jumped straight to Planning (or Morning -> Planning), print context
                    if driver.state.phase == GamePhase::TacticalPlanning {
                        write!(current_buffer, "{}", format_planning_context(&driver.state))
                            .unwrap();
                    } else {
                        // Might be in MorningReport if something weird happened, but usually Driver stabilizes to Planning.
                        if driver.state.phase == GamePhase::MorningReport
                            && let Some(card) = &driver.state.latest_event
                        {
                            writeln!(
                                current_buffer,
                                "[EVENT] {} - {}",
                                card.title, card.description
                            )
                            .unwrap();
                        }
                    }
                } else if driver.state.phase != prev_phase {
                    // Phase changed but same round (e.g. Planning -> Execution -> Planning?? No, Driver stabilizes out of Execution)
                    // Driver.stabilize() only stops at TacticalPlanning or EndGame.
                    // So if we see a phase change here, it means we went Planning -> Planning (via Execution/Morning).
                    // But that would increment turn count usually.
                    // Unless we went from Planning -> GameOver.

                    if driver.state.phase == GamePhase::GameOver
                        || driver.state.phase == GamePhase::Victory
                    {
                        writeln!(
                            current_buffer,
                            "\n=== GAME ENDED: {:?} ===",
                            driver.state.phase
                        )
                        .unwrap();
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

    writeln!(
        current_buffer,
        "\n=== LAST PHASE: {:?} ===",
        driver.state.phase
    )
    .unwrap();
    rounds_output.push(current_buffer);

    rounds_output
}

pub fn print_trajectory(initial_state: GameState, path: Vec<(PlayerId, GameAction)>) {
    let rounds = format_trajectory(initial_state, path);
    for r in rounds {
        print!("{}", r);
    }
}

fn format_planning_context(state: &GameState) -> String {
    let mut out = String::new();

    // 1. Morning Report Info (Event) - Recovered from state if possible
    // Note: In strict Driver mode, we might miss the MorningReport event print if we don't catch it during stabilization.
    // Ideally Driver would return a log of events.
    // For now, we print what we see in the state.
    if let Some(card) = &state.latest_event {
        writeln!(out, "[EVENT] {} - {}", card.title, card.description).unwrap();
    }
    if !state.active_situations.is_empty() {
        let names: Vec<String> = state
            .active_situations
            .iter()
            .map(|c| c.title.clone())
            .collect();
        writeln!(out, "[ACTIVE] {}", names.join(", ")).unwrap();
    }

    // 2. Enemy Intent
    if let Some(attack) = &state.enemy.next_attack {
        writeln!(
            out,
            "[ENEMY] {} targets Room {} with {:?}",
            state.enemy.name, attack.target_room, attack.effect
        )
        .unwrap();
    }

    // 3. Hazards
    let mut fire_rooms = Vec::new();
    let mut water_rooms = Vec::new();

    let room_ids: Vec<u32> = state.map.rooms.keys().collect();

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

    // 4. Players
    let mut players_info = Vec::new();
    let pids: Vec<String> = state.players.keys().cloned().collect();
    for pid in pids {
        if let Some(p) = state.players.get(&pid) {
            let inv_str = if p.inventory.is_empty() {
                "".to_owned()
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
