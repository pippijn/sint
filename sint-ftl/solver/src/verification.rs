use crate::scoring::beam::ScoreAccumulator;
use serde::{Deserialize, Serialize};
use sint_core::GameError;
use sint_core::logic::GameLogic;
use sint_core::types::{Action, GameAction, GamePhase, GameState, ItemType};

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

pub type RoundActions = Vec<(String, GameAction)>;
pub type ParsedSolution = (Vec<RoundActions>, Option<u64>, Option<usize>);

impl VerificationResult {
    pub fn failure_summary(&self) -> Option<String> {
        if self.success {
            return None;
        }

        let mut out = String::new();
        out.push_str("\n=== FAILURE SUMMARY ===\n");
        let state = &self.final_state;
        out.push_str(&format!("Round: {}\n", state.turn_count));
        out.push_str(&format!("Phase: {:?}\n", state.phase));

        if let Some((pid, action)) = &self.failed_action {
            out.push_str(&format!("Failed Action: {} performs {:?}\n", pid, action));
            if let Some(error) = &self.error {
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
            }
        } else {
            // No specific action failed, but the overall result is not Success
            if state.phase == GamePhase::GameOver {
                out.push_str("Result: DEFEAT (Ship Destroyed)\n");
            } else {
                out.push_str("Result: INCOMPLETE (Simulation ended without Victory)\n");
            }
            if let Some(error) = &self.error {
                out.push_str(&format!("Error: {}\n", error));
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
        let room_ids: Vec<u32> = state.map.rooms.keys().collect();
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
        let pids: Vec<String> = state.players.keys().cloned().collect();
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

pub fn parse_game_action(cmd: &str) -> GameAction {
    let cmd = cmd.trim();
    if cmd.starts_with("Move") {
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
    } else if cmd.starts_with("PickUp") {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let item_type = if parts.len() > 1 {
            match parts[1] {
                "Peppernut" => ItemType::Peppernut,
                "Extinguisher" => ItemType::Extinguisher,
                "Keychain" => ItemType::Keychain,
                "Wheelbarrow" => ItemType::Wheelbarrow,
                "Mitre" => ItemType::Mitre,
                _ => ItemType::Peppernut,
            }
        } else {
            ItemType::Peppernut
        };
        GameAction::PickUp { item_type }
    } else if cmd.starts_with("Drop") {
        let idx = cmd.split_whitespace().nth(1).unwrap().parse().unwrap();
        GameAction::Drop { item_index: idx }
    } else if cmd == "Pass" {
        GameAction::Pass
    } else if cmd == "Ready" || cmd == "VoteReady" {
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
    }
}

pub fn parse_solution_text(text: &str) -> ParsedSolution {
    let mut rounds = Vec::new();
    let mut current_round = Vec::new();
    let mut seed = None;
    let mut players = None;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with('#') {
            if line.to_lowercase().contains("round") && !current_round.is_empty() {
                rounds.push(current_round);
                current_round = Vec::new();
            }
            continue;
        }

        // Check for Meta Commands: "SEED 12345" or "PLAYERS 2"
        if line.starts_with("SEED ")
            && let Ok(val) = line.replace("SEED ", "").trim().parse::<u64>()
        {
            seed = Some(val);
            continue;
        }
        if line.starts_with("PLAYERS ")
            && let Ok(val) = line.replace("PLAYERS ", "").trim().parse::<usize>()
        {
            players = Some(val);
            continue;
        }

        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() != 2 {
            continue;
        }

        let pid = parts[0].trim().to_owned();
        let cmd = parts[1].trim();

        let action = parse_game_action(cmd);

        current_round.push((pid, action));
    }

    if !current_round.is_empty() {
        rounds.push(current_round);
    }
    // If no explicit rounds found but actions exist, treat as one round (legacy/fallback)
    // Or if valid use case requires implicit single round.

    (rounds, seed, players)
}

pub fn run_verification(
    initial_state: GameState,
    user_rounds: Vec<RoundActions>,
) -> VerificationResult {
    let mut state = initial_state;
    let mut full_history = Vec::new();
    let mut scorer = ScoreAccumulator::new();
    let mut last_round = state.turn_count;

    for round_actions in user_rounds.into_iter() {
        if state.phase == GamePhase::GameOver || state.phase == GamePhase::Victory {
            break;
        }

        // 1. Advance through Non-Interactive Phases (MorningReport, etc.)
        // We expect the state to eventually reach TacticalPlanning or GameOver
        let mut loop_safety = 0;
        while state.phase != GamePhase::TacticalPlanning
            && state.phase != GamePhase::GameOver
            && state.phase != GamePhase::Victory
        {
            loop_safety += 1;
            if loop_safety > 100 {
                return VerificationResult {
                    success: false,
                    history: full_history,
                    final_state: state.clone(),
                    error: Some(GameError::InvalidAction("Stuck in phase transition".into())),
                    failed_action: None,
                    score: scorer.finalize(&state),
                };
            }

            // Auto-Vote Ready for everyone in non-planning phases
            loop {
                if state.phase == GamePhase::TacticalPlanning
                    || state.phase == GamePhase::GameOver
                    || state.phase == GamePhase::Victory
                {
                    break;
                }

                // Find deterministic next player
                let next_unready = state
                    .players
                    .values()
                    .filter(|p| !p.is_ready)
                    .map(|p| p.id.clone())
                    .min();

                if let Some(pid) = next_unready {
                    let act = GameAction::VoteReady { ready: true };
                    let core_act = Action::Game(act.clone());
                    if let Ok(s) = GameLogic::apply_action(state.clone(), &pid, core_act, None) {
                        state = s;
                        full_history.push((pid, act));
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            // Check for round transition
            if state.turn_count > last_round {
                scorer.on_round_end(&state);
                last_round = state.turn_count;
            }
        }

        if state.phase == GamePhase::GameOver || state.phase == GamePhase::Victory {
            break;
        }

        // 2. We are now in TacticalPlanning (presumably). Apply actions for this block.
        // Record the round number at start of block
        let round_start_turn = state.turn_count;

        for (pid, action) in round_actions {
            // Check if we accidentally drifted to next round prematurely?
            // If state.turn_count > round_start_turn, that means the previous action triggered a round end.
            // But we still have actions in the block! This violates "Block = One Round".
            // However, maybe the user wants to supply actions for Morning phase?
            // But we auto-skipped Morning.
            // So strictly speaking, all actions in this block should belong to round_start_turn.
            if state.turn_count > round_start_turn {
                return VerificationResult {
                    success: false,
                    history: full_history,
                    final_state: state.clone(),
                    error: Some(GameError::InvalidAction(format!(
                        "Round advanced to {} while still executing actions for block representing Round {}. Block has extra actions.",
                        state.turn_count, round_start_turn
                    ))),
                    failed_action: Some((pid, action)),
                    score: scorer.finalize(&state),
                };
            }

            let core_act = Action::Game(action.clone());
            match GameLogic::apply_action(state.clone(), &pid, core_act, None) {
                Ok(s) => {
                    full_history.push((pid.clone(), action.clone()));
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
        }

        // 3. Block finished. Ensure round completion.
        if state.turn_count == round_start_turn && state.phase == GamePhase::TacticalPlanning {
            // Ensure all AP consumed by active players
            let players_with_ap: Vec<String> = state
                .players
                .values()
                .filter(|p| p.ap > 0 && !p.is_ready)
                .map(|p| p.id.clone())
                .collect();

            if !players_with_ap.is_empty() {
                return VerificationResult {
                    success: false,
                    history: full_history,
                    final_state: state.clone(),
                    error: Some(GameError::InvalidAction(format!(
                        "Round {} block finished, but players still have AP: {:?}",
                        round_start_turn, players_with_ap
                    ))),
                    failed_action: None,
                    score: scorer.finalize(&state),
                };
            }

            // Auto-ready remaining players (who must have 0 AP) to trigger phase transition
            loop {
                if state.phase != GamePhase::TacticalPlanning {
                    break;
                }

                let next_unready = state
                    .players
                    .values()
                    .filter(|p| !p.is_ready)
                    .map(|p| p.id.clone())
                    .min();

                if let Some(pid) = next_unready {
                    let act = GameAction::VoteReady { ready: true };
                    let core_act = Action::Game(act.clone());
                    if let Ok(s) = GameLogic::apply_action(state.clone(), &pid, core_act, None) {
                        state = s;
                        full_history.push((pid, act));
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        // Verify phase transition
        if state.turn_count != round_start_turn || state.phase != GamePhase::TacticalPlanning {
            // Round or Phase advanced successfully
            if state.turn_count > last_round {
                scorer.on_round_end(&state);
                last_round = state.turn_count;
            }
        }
    }

    // Check outcome
    let is_victory = state.phase == GamePhase::Victory;
    VerificationResult {
        success: is_victory,
        history: full_history,
        final_state: state.clone(),
        error: None,
        failed_action: None,
        score: scorer.finalize(&state),
    }
}
