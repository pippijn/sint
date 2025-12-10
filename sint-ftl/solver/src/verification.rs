use crate::scoring::ScoreAccumulator;
use serde::{Deserialize, Serialize};
use sint_core::logic::GameLogic;
use sint_core::types::{Action, GameAction, GamePhase, GameState, ItemType};
use sint_core::GameError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub success: bool,
    pub history: Vec<(String, Action)>,
    pub final_state: GameState,
    pub error: Option<GameError>,
    pub failed_action: Option<(String, Action)>,
    pub score: f64,
}

impl VerificationResult {
    pub fn failure_summary(&self) -> Option<String> {
        if self.success {
            return None;
        }
        let (pid, action) = self.failed_action.as_ref()?;
        let error = self.error.as_ref()?;
        let state = &self.final_state;

        let mut out = String::new();
        out.push_str("\n=== FAILURE SUMMARY ===\n");
        out.push_str(&format!("Round: {}\n", state.turn_count));
        out.push_str(&format!("Phase: {:?}\n", state.phase));
        out.push_str(&format!("Failed Action: {} performs {:?}\n", pid, action));
        out.push_str(&format!("Error: {}\n", error));

        if let GameError::NotEnoughAP = error {
            let other_players_with_ap: Vec<_> = state
                .players
                .values()
                .filter(|p| p.id != *pid && p.ap > 0)
                .collect();
            if other_players_with_ap.len() == 1 {
                let p = other_players_with_ap[0];
                out.push_str(&format!(
                    "Hint: Round not over. {} still has {} AP.\n",
                    p.id, p.ap
                ));
            }
        }

        out.push_str("\n-- State Context --\n");
        out.push_str(&format!(
            "Hull: {} | Enemy: {} ({} HP)\n",
            state.hull_integrity, state.enemy.name, state.enemy.hp
        ));

        out.push_str("Active Situations:\n");
        for card in &state.active_situations {
            out.push_str(&format!("  - {} ({:?})\n", card.title, card.id));
        }

        out.push_str("Room Items:\n");
        let mut room_ids: Vec<u32> = state.map.rooms.keys().cloned().collect();
        room_ids.sort();
        for rid in &room_ids {
            if let Some(room) = state.map.rooms.get(rid) {
                if !room.items.is_empty() {
                    out.push_str(&format!("  Room {}: {:?}\n", rid, room.items));
                }
            }
        }

        out.push_str("Hazards:\n");
        for rid in &room_ids {
            if let Some(room) = state.map.rooms.get(rid) {
                if !room.hazards.is_empty() {
                    out.push_str(&format!("  Room {}: {:?}\n", rid, room.hazards));
                }
            }
        }

        out.push_str("Players:\n");
        let mut pids: Vec<String> = state.players.keys().cloned().collect();
        pids.sort();
        for p_id in pids {
            if let Some(p) = state.players.get(&p_id) {
                out.push_str(&format!(
                    "  {}: Room {} | AP {} | HP {} | Inv {:?} | Status {:?}\n",
                    p_id, p.room_id, p.ap, p.hp, p.inventory, p.status
                ));
            }
        }
        out.push_str("=======================\n");
        Some(out)
    }
}

pub fn parse_solution_text(text: &str) -> (Vec<(String, Action)>, Option<u64>, Option<usize>) {
    let mut actions = Vec::new();
    let mut seed = None;
    let mut players = None;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Check for Meta Commands: "SEED 12345" or "PLAYERS 2"
        if line.starts_with("SEED ") {
            if let Ok(val) = line.replace("SEED ", "").trim().parse::<u64>() {
                seed = Some(val);
                continue;
            }
        }
        if line.starts_with("PLAYERS ") {
            if let Ok(val) = line.replace("PLAYERS ", "").trim().parse::<usize>() {
                players = Some(val);
                continue;
            }
        }

        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() != 2 {
            continue;
        }

        let pid = parts[0].trim().to_string();
        let cmd = parts[1].trim();

        let action = if cmd.starts_with("Move") {
            let target: u32 = cmd.split_whitespace().nth(1).unwrap().parse().unwrap();
            Action::Game(GameAction::Move { to_room: target })
        } else if cmd == "Bake" {
            Action::Game(GameAction::Bake)
        } else if cmd == "Shoot" {
            Action::Game(GameAction::Shoot)
        } else if cmd.starts_with("Throw") {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            let target = parts[1].to_string();
            let idx = parts[2].parse().unwrap();
            Action::Game(GameAction::Throw {
                target_player: target,
                item_index: idx,
            })
        } else if cmd == "Extinguish" {
            Action::Game(GameAction::Extinguish)
        } else if cmd == "Repair" {
            Action::Game(GameAction::Repair)
        } else if cmd == "PickUp" {
            Action::Game(GameAction::PickUp {
                item_type: ItemType::Peppernut,
            })
        } else if cmd.starts_with("Drop") {
            let idx = cmd.split_whitespace().nth(1).unwrap().parse().unwrap();
            Action::Game(GameAction::Drop { item_index: idx })
        } else if cmd == "Pass" {
            Action::Game(GameAction::Pass)
        } else if cmd == "Ready" {
            Action::Game(GameAction::VoteReady { ready: true })
        } else if cmd == "RaiseShields" {
            Action::Game(GameAction::RaiseShields)
        } else if cmd == "EvasiveManeuvers" {
            Action::Game(GameAction::EvasiveManeuvers)
        } else if cmd == "Lookout" {
            Action::Game(GameAction::Lookout)
        } else if cmd == "Interact" {
            Action::Game(GameAction::Interact)
        } else if cmd.starts_with("Revive") {
            let target = cmd.split_whitespace().nth(1).unwrap().to_string();
            Action::Game(GameAction::Revive {
                target_player: target,
            })
        } else if cmd.starts_with("FirstAid") {
            let target = cmd.split_whitespace().nth(1).unwrap().to_string();
            Action::Game(GameAction::FirstAid {
                target_player: target,
            })
        } else if cmd.starts_with("Chat") {
            let msg = cmd.replace("Chat ", "").trim().to_string();
            Action::Game(GameAction::Chat { message: msg })
        } else {
            // Panic or ignore? The original code panicked.
            // Let's print error and continue or panic.
            // For library code, panic is bad. But this is a parser helper.
            panic!("Unknown command: {}", cmd);
        };

        actions.push((pid, action));
    }
    (actions, seed, players)
}

pub fn run_verification(
    initial_state: GameState,
    user_actions: Vec<(String, Action)>,
) -> VerificationResult {
    let mut state = initial_state;
    let mut full_history = Vec::new();
    let mut action_iter = user_actions.into_iter();

    let mut scorer = ScoreAccumulator::new();
    let mut last_round = state.turn_count;

    // Loop until Game Over or Actions exhausted
    loop {
        // Scoring: Check round transition
        if state.turn_count > last_round {
            // A round has passed. Capture state for scoring.
            scorer.on_round_end(&state);
            last_round = state.turn_count;
        }

        if state.phase == GamePhase::GameOver || state.phase == GamePhase::Victory {
            break;
        }

        // Auto-Advance Phases
        if state.phase != GamePhase::TacticalPlanning {
            // Just Vote Ready for everyone
            let pids: Vec<String> = state.players.keys().cloned().collect();
            for pid in pids {
                let act = Action::Game(GameAction::VoteReady { ready: true });
                match GameLogic::apply_action(state.clone(), &pid, act.clone(), None) {
                    Ok(s) => {
                        state = s;
                        full_history.push((pid, act));
                    }
                    Err(_) => break, // Should not happen
                }
            }
            continue;
        }

        if let Some((pid, action)) = action_iter.next() {
            match GameLogic::apply_action(state.clone(), &pid, action.clone(), None) {
                Ok(mut s) => {
                    full_history.push((pid.clone(), action.clone()));

                    // Auto-Ready Logic
                    if s.phase == GamePhase::TacticalPlanning {
                        if let Some(p) = s.players.get(&pid) {
                            if p.ap == 0 && !p.is_ready {
                                let ready_act = Action::Game(GameAction::VoteReady { ready: true });
                                match GameLogic::apply_action(
                                    s.clone(),
                                    &pid,
                                    ready_act.clone(),
                                    None,
                                ) {
                                    Ok(next_s) => {
                                        s = next_s;
                                        full_history.push((pid.clone(), ready_act));
                                    }
                                    Err(_) => {} // Auto-ready failed
                                }
                            }
                        }
                    }
                    state = s;
                }
                Err(e) => {
                    return VerificationResult {
                        success: false,
                        history: full_history,
                        final_state: state.clone(),
                        error: Some(e),
                        failed_action: Some((pid, action)),
                        score: scorer.finalize(&state),
                    };
                }
            }
        } else {
            // No more user actions
            break;
        }
    }

    // Capture final round if not captured (e.g. game ended mid-round or at end of round)
    if state.turn_count >= last_round {
        scorer.on_round_end(&state);
    }

    VerificationResult {
        success: true,
        history: full_history,
        final_state: state.clone(),
        error: None,
        failed_action: None,
        score: scorer.finalize(&state),
    }
}
