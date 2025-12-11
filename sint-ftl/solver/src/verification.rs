use crate::scoring::ScoreAccumulator;
use serde::{Deserialize, Serialize};
use sint_core::logic::GameLogic;
use sint_core::types::{Action, GameAction, GamePhase, GameState, ItemType};
use sint_core::GameError;

fn get_hazard_emoji(h: &sint_core::types::HazardType) -> &'static str {
    match h {
        sint_core::types::HazardType::Fire => "🔥",
        sint_core::types::HazardType::Water => "💧",
    }
}

fn get_item_emoji(i: &ItemType) -> &'static str {
    match i {
        ItemType::Peppernut => "🍪",
        ItemType::Extinguisher => "🧯",
        ItemType::Keychain => "🔑",
        ItemType::Wheelbarrow => "🛒",
        ItemType::Mitre => "🧢",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub success: bool,
    pub history: Vec<(String, GameAction)>,
    pub final_state: GameState,
    pub error: Option<GameError>,
    pub failed_action: Option<(String, GameAction)>,
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
        // Action is now GameAction, so formatting {:?} gives the clean output directly
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

        out.push_str("Map State:\n");
        let mut room_ids: Vec<u32> = state.map.rooms.keys().cloned().collect();
        room_ids.sort();
        for rid in &room_ids {
            if let Some(room) = state.map.rooms.get(rid) {
                let sys = room
                    .system
                    .map(|s| format!("{:?}", s))
                    .unwrap_or("Empty".to_owned());

                let hazards_str: String = room.hazards.iter().map(get_hazard_emoji).collect();

                let items_str: String = room.items.iter().map(get_item_emoji).collect();

                out.push_str(&format!(
                    "  Room {} ({}): System={} | Neighbors={:?} | Hazards=[{}] | Items=[{}]\n",
                    rid, room.name, sys, room.neighbors, hazards_str, items_str
                ));
            }
        }

        out.push_str("Players:\n");
        let mut pids: Vec<String> = state.players.keys().cloned().collect();
        pids.sort();
        for p_id in pids {
            if let Some(p) = state.players.get(&p_id) {
                let inv_str: String = p.inventory.iter().map(get_item_emoji).collect();

                out.push_str(&format!(
                    "  {}: Room {} | AP {} | HP {} | Inv [{}] | Status {:?}\n",
                    p_id, p.room_id, p.ap, p.hp, inv_str, p.status
                ));
            }
        }
        out.push_str("=======================\n");
        Some(out)
    }
}

pub fn parse_solution_text(text: &str) -> (Vec<(String, GameAction)>, Option<u64>, Option<usize>) {
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

        let pid = parts[0].trim().to_owned();
        let cmd = parts[1].trim();

        let action = if cmd.starts_with("Move") {
            let target: u32 = cmd.split_whitespace().nth(1).unwrap().parse().unwrap();
            GameAction::Move { to_room: target }
        } else if cmd == "Bake" {
            GameAction::Bake
        } else if cmd == "Shoot" {
            GameAction::Shoot
        } else if cmd.starts_with("Throw") {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            let target = parts[1].to_owned();
            let idx = parts[2].parse().unwrap();
            GameAction::Throw {
                target_player: target,
                item_index: idx,
            }
        } else if cmd == "Extinguish" {
            GameAction::Extinguish
        } else if cmd == "Repair" {
            GameAction::Repair
        } else if cmd == "PickUp" {
            GameAction::PickUp {
                item_type: ItemType::Peppernut,
            }
        } else if cmd.starts_with("Drop") {
            let idx = cmd.split_whitespace().nth(1).unwrap().parse().unwrap();
            GameAction::Drop { item_index: idx }
        } else if cmd == "Pass" {
            GameAction::Pass
        } else if cmd == "Ready" {
            GameAction::VoteReady { ready: true }
        } else if cmd == "RaiseShields" {
            GameAction::RaiseShields
        } else if cmd == "EvasiveManeuvers" {
            GameAction::EvasiveManeuvers
        } else if cmd == "Lookout" {
            GameAction::Lookout
        } else if cmd == "Interact" {
            GameAction::Interact
        } else if cmd.starts_with("Revive") {
            let target = cmd.split_whitespace().nth(1).unwrap().to_owned();
            GameAction::Revive {
                target_player: target,
            }
        } else if cmd.starts_with("FirstAid") {
            let target = cmd.split_whitespace().nth(1).unwrap().to_owned();
            GameAction::FirstAid {
                target_player: target,
            }
        } else if cmd.starts_with("Chat") {
            let msg = cmd.replace("Chat ", "").trim().to_owned();
            GameAction::Chat { message: msg }
        } else {
            panic!("Unknown command: {}", cmd);
        };

        actions.push((pid, action));
    }
    (actions, seed, players)
}

pub fn run_verification(
    initial_state: GameState,
    user_actions: Vec<(String, GameAction)>,
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
                let act = GameAction::VoteReady { ready: true };
                // Wrap in Action::Game for core logic
                let core_act = Action::Game(act.clone());
                match GameLogic::apply_action(state.clone(), &pid, core_act, None) {
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
            // Wrap in Action::Game for core logic
            let core_act = Action::Game(action.clone());
            match GameLogic::apply_action(state.clone(), &pid, core_act, None) {
                Ok(mut s) => {
                    full_history.push((pid.clone(), action.clone()));

                    // Auto-Ready Logic
                    if s.phase == GamePhase::TacticalPlanning {
                        if let Some(p) = s.players.get(&pid) {
                            if p.ap == 0 && !p.is_ready {
                                let ready_act = GameAction::VoteReady { ready: true };
                                let core_ready = Action::Game(ready_act.clone());
                                match GameLogic::apply_action(s.clone(), &pid, core_ready, None) {
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
