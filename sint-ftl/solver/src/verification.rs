use crate::scoring::ScoreDetails;
use crate::scoring::beam::{BeamScoringWeights, ScoreAccumulator, calculate_score};
use crate::scoring::rhea::{RheaScoringWeights, score_rhea};
use crate::scoring::rl::{RlScoringWeights, score_rl};
use serde::{Deserialize, Serialize};
use sint_core::GameError;
use sint_core::logic::GameLogic;
use sint_core::logic::pathfinding::MapDistances;
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
    pub beam_score: f64,
    pub rhea_score: f64,
    pub rl_score: f64,
    pub rl_details: ScoreDetails,
}

pub type RoundActions = Vec<(String, GameAction)>;

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

pub fn run_verification(
    initial_state: GameState,
    user_rounds: Vec<RoundActions>,
) -> VerificationResult {
    let mut state = initial_state;
    let mut full_history = Vec::new();
    let mut scorer = ScoreAccumulator::new();
    let mut last_round = state.turn_count;

    let mut parent_state = state.clone();
    let mut last_round_actions: Vec<(String, GameAction)> = Vec::new();
    let mut error = None;
    let mut failed_action = None;

    'outer: for round_actions in user_rounds.into_iter() {
        if state.phase == GamePhase::GameOver || state.phase == GamePhase::Victory {
            break;
        }

        // 1. Advance through Non-Interactive Phases (MorningReport, etc.)
        let mut loop_safety = 0;
        while state.phase != GamePhase::TacticalPlanning
            && state.phase != GamePhase::GameOver
            && state.phase != GamePhase::Victory
        {
            loop_safety += 1;
            if loop_safety > 100 {
                error = Some(GameError::InvalidAction("Stuck in phase transition".into()));
                break 'outer;
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
        let round_start_turn = state.turn_count;
        let mut current_round_actions = Vec::new();

        for (pid, action) in round_actions {
            if state.turn_count > round_start_turn {
                error = Some(GameError::InvalidAction(format!(
                    "Round advanced to {} while still executing actions for block representing Round {}. Block has extra actions.",
                    state.turn_count, round_start_turn
                )));
                failed_action = Some((pid, action));
                break 'outer;
            }

            let core_act = Action::Game(action.clone());
            parent_state = state.clone();
            match GameLogic::apply_action(state.clone(), &pid, core_act, None) {
                Ok(s) => {
                    full_history.push((pid.clone(), action.clone()));
                    current_round_actions.push((pid.clone(), action.clone()));
                    state = s;
                }
                Err(e) => {
                    error = Some(e);
                    failed_action = Some((pid, action));
                    break 'outer;
                }
            }
        }
        last_round_actions = current_round_actions;

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
                error = Some(GameError::InvalidAction(format!(
                    "Round {} block finished, but players still have AP: {:?}",
                    round_start_turn, players_with_ap
                )));
                break 'outer;
            }

            // Auto-ready remaining players
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

    // Scoring
    let rhea_weights = RheaScoringWeights::default();
    let rhea_score = score_rhea(&state, &rhea_weights).total;

    let beam_weights = BeamScoringWeights::default();
    let distances = MapDistances::new(&state.map);
    let borrowed_history: Vec<&(sint_core::types::PlayerId, GameAction)> =
        last_round_actions.iter().collect();

    let beam_score = calculate_score(
        &parent_state,
        &state,
        &borrowed_history,
        &beam_weights,
        &distances,
    )
    .total;

    let rl_weights = RlScoringWeights::default();
    let rl_details = score_rl(&parent_state, &state, &borrowed_history, &rl_weights);
    let rl_score = rl_details.total;

    // Check outcome
    let is_victory = state.phase == GamePhase::Victory;
    VerificationResult {
        success: is_victory && error.is_none(),
        history: full_history,
        final_state: state.clone(),
        error,
        failed_action,
        score: scorer.finalize(&state),
        beam_score,
        rhea_score,
        rl_score,
        rl_details,
    }
}

pub fn run_verification_linear(
    initial_state: GameState,
    actions: Vec<(String, GameAction)>,
) -> VerificationResult {
    let mut state = initial_state;
    let mut full_history = Vec::new();
    let mut scorer = ScoreAccumulator::new();
    let mut last_round = state.turn_count;

    let mut parent_state = state.clone();
    let mut current_round_actions: Vec<(String, GameAction)> = Vec::new();
    let mut error = None;
    let mut failed_action = None;

    for (pid, action) in actions {
        if state.phase == GamePhase::GameOver || state.phase == GamePhase::Victory {
            break;
        }

        // 1. Auto-advance through non-interactive phases
        let mut loop_safety = 0;
        while state.phase != GamePhase::TacticalPlanning
            && state.phase != GamePhase::GameOver
            && state.phase != GamePhase::Victory
        {
            loop_safety += 1;
            if loop_safety > 100 {
                error = Some(GameError::InvalidAction("Stuck in phase transition".into()));
                break;
            }

            let next_unready = state
                .players
                .values()
                .filter(|p| !p.is_ready)
                .map(|p| p.id.clone())
                .min();

            if let Some(p_id) = next_unready {
                let act = GameAction::VoteReady { ready: true };
                let core_act = Action::Game(act.clone());
                if let Ok(s) = GameLogic::apply_action(state.clone(), &p_id, core_act, None) {
                    state = s;
                    full_history.push((p_id, act));
                } else {
                    break;
                }
            } else {
                break;
            }

            if state.turn_count > last_round {
                scorer.on_round_end(&state);
                last_round = state.turn_count;
                current_round_actions.clear();
            }
        }
        if error.is_some() {
            break;
        }

        if state.phase == GamePhase::GameOver || state.phase == GamePhase::Victory {
            break;
        }

        parent_state = state.clone();

        // 2. Apply the action
        let core_act = Action::Game(action.clone());
        match GameLogic::apply_action(state.clone(), &pid, core_act, None) {
            Ok(mut s) => {
                // In linear mode, we want immediate resolution for Tactical actions
                // to provide dense feedback.
                if s.phase == GamePhase::TacticalPlanning {
                    sint_core::logic::resolution::resolve_proposal_queue(&mut s, false);
                }

                full_history.push((pid.clone(), action.clone()));
                current_round_actions.push((pid.clone(), action.clone()));
                state = s;
            }
            Err(e) => {
                error = Some(e);
                failed_action = Some((pid, action));
                break;
            }
        }

        // If no one has AP left, auto-ready everyone to advance phase
        if state.phase == GamePhase::TacticalPlanning {
            let no_ap_anywhere = state.players.values().all(|p| p.ap == 0 || p.is_ready);
            if no_ap_anywhere {
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
                        let act = Action::Game(GameAction::VoteReady { ready: true });
                        if let Ok(s) = GameLogic::apply_action(state.clone(), &pid, act, None) {
                            state = s;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        if state.turn_count > last_round {
            scorer.on_round_end(&state);
            last_round = state.turn_count;
            current_round_actions.clear();
        }
    }

    // 3. Final auto-advance through non-interactive phases to reach next TacticalPlanning
    let mut loop_safety = 0;
    while error.is_none()
        && state.phase != GamePhase::TacticalPlanning
        && state.phase != GamePhase::GameOver
        && state.phase != GamePhase::Victory
    {
        loop_safety += 1;
        if loop_safety > 100 {
            error = Some(GameError::InvalidAction("Stuck in phase transition".into()));
            break;
        }

        let next_unready = state
            .players
            .values()
            .filter(|p| !p.is_ready)
            .map(|p| p.id.clone())
            .min();

        if let Some(p_id) = next_unready {
            let act = GameAction::VoteReady { ready: true };
            let core_act = Action::Game(act.clone());
            if let Ok(s) = GameLogic::apply_action(state.clone(), &p_id, core_act, None) {
                state = s;
                full_history.push((p_id, act));
            } else {
                break;
            }
        } else {
            break;
        }

        if state.turn_count > last_round {
            scorer.on_round_end(&state);
        }
    }

    // 4. Auto-ready everyone with 0 AP in TacticalPlanning and advance if possible
    let mut loop_safety = 0;
    while error.is_none()
        && (state.phase == GamePhase::TacticalPlanning
            || (state.phase != GamePhase::GameOver && state.phase != GamePhase::Victory))
    {
        loop_safety += 1;
        if loop_safety > 200 {
            error = Some(GameError::InvalidAction("Stuck in phase transition".into()));
            break;
        }

        if state.phase == GamePhase::TacticalPlanning {
            let next_unready_0ap = state
                .players
                .values()
                .filter(|p| !p.is_ready && p.ap == 0)
                .map(|p| p.id.clone())
                .min();

            if let Some(pid) = next_unready_0ap {
                let act = Action::Game(GameAction::VoteReady { ready: true });
                if let Ok(s) = GameLogic::apply_action(state.clone(), &pid, act, None) {
                    state = s;
                    continue; // Check again
                }
            }
        }

        // If not in TacticalPlanning or everyone with 0AP is ready, try to advance next_unready generally
        if state.phase != GamePhase::TacticalPlanning
            && state.phase != GamePhase::GameOver
            && state.phase != GamePhase::Victory
        {
            let next_unready = state
                .players
                .values()
                .filter(|p| !p.is_ready)
                .map(|p| p.id.clone())
                .min();

            if let Some(pid) = next_unready {
                let act = Action::Game(GameAction::VoteReady { ready: true });
                if let Ok(s) = GameLogic::apply_action(state.clone(), &pid, act, None) {
                    state = s;
                } else {
                    break;
                }
            } else {
                break;
            }
        } else {
            // In TacticalPlanning with no 0AP players unready, or terminal state
            break;
        }

        if state.turn_count > last_round {
            scorer.on_round_end(&state);
            last_round = state.turn_count;
        }
    }

    // Final scoring
    let rhea_weights = RheaScoringWeights::default();
    let rhea_score = score_rhea(&state, &rhea_weights).total;

    let beam_weights = BeamScoringWeights::default();
    let distances = MapDistances::new(&state.map);
    let borrowed_history: Vec<&(sint_core::types::PlayerId, GameAction)> =
        current_round_actions.iter().collect();

    let beam_score = calculate_score(
        &parent_state,
        &state,
        &borrowed_history,
        &beam_weights,
        &distances,
    )
    .total;

    let rl_weights = RlScoringWeights::default();
    let rl_details = score_rl(&parent_state, &state, &borrowed_history, &rl_weights);
    let rl_score = rl_details.total;

    let is_victory = state.phase == GamePhase::Victory;
    VerificationResult {
        success: is_victory && error.is_none(),
        history: full_history,
        final_state: state.clone(),
        error,
        failed_action,
        score: scorer.finalize(&state),
        beam_score,
        rhea_score,
        rl_score,
        rl_details,
    }
}
