use super::cards;
use super::pathfinding::{find_path, get_player_projected_room};
use super::resolution;
use crate::logic::GameError;
use crate::types::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use uuid::Uuid;

pub fn apply_action(
    mut state: GameState,
    player_id: &str,
    action: Action,
) -> Result<GameState, GameError> {
    // 1. Handle Join & FullSync (Special Cases)
    if let Action::Join { name } = &action {
        if state.players.contains_key(player_id) {
            return Ok(state);
        }
        if state.players.values().any(|p| p.name == *name) {
            return Err(GameError::InvalidAction("Name already taken".to_string()));
        }
        state.players.insert(
            player_id.to_string(),
            Player {
                id: player_id.to_string(),
                name: name.clone(),
                room_id: 3,
                hp: 3,
                ap: 2,
                inventory: vec![],
                status: vec![],
                is_ready: false,
            },
        );
        state.sequence_id += 1;
        return Ok(state);
    }

    if let Action::FullSync { state_json } = &action {
        // Replace state completely
        match serde_json::from_str::<GameState>(state_json) {
            Ok(new_state) => return Ok(new_state),
            Err(e) => return Err(GameError::InvalidAction(format!("Bad Sync: {}", e))),
        }
    }

    if let Action::SetName { name } = &action {
        if state.phase != GamePhase::Lobby {
            return Err(GameError::InvalidAction(
                "Cannot change name after game start".to_string(),
            ));
        }
        if state
            .players
            .values()
            .any(|p| p.name == *name && p.id != player_id)
        {
            return Err(GameError::InvalidAction("Name already taken".to_string()));
        }

        if let Some(p) = state.players.get_mut(player_id) {
            p.name = name.clone();
        } else {
            return Err(GameError::PlayerNotFound);
        }
        state.sequence_id += 1;
        return Ok(state);
    }

    // Phase Restriction: Gameplay actions only in TacticalPlanning
    if state.phase != GamePhase::TacticalPlanning {
        match action {
            Action::Chat { .. } | Action::VoteReady { .. } => {}
            _ => {
                return Err(GameError::InvalidAction(format!(
                    "Cannot act during {:?}",
                    state.phase
                )))
            }
        }
    }

    // 2. Validate AP (unless it's free)
    let base_cost = action_cost(&state, &action);
    let player = state
        .players
        .get_mut(player_id)
        .ok_or(GameError::PlayerNotFound)?;

    if player.ap < base_cost {
        return Err(GameError::NotEnoughAP);
    }

    // 3. Execute OR Queue Logic
    let mut is_immediate = false;
    let mut ap_deducted = false;

    match &action {
        Action::Chat { message } => {
            is_immediate = true;
            // C02: Static Noise
            let static_noise = state.active_situations.iter().any(|c| c.id == "C02");
            if static_noise {
                let has_alpha = message.chars().any(|c| c.is_alphabetic());
                if has_alpha {
                    return Err(GameError::Silenced);
                }
            }

            state.chat_log.push(ChatMessage {
                sender: player_id.to_string(),
                text: message.clone(),
                timestamp: 0,
            });
        }
        Action::VoteReady { ready } => {
            is_immediate = true;
            let p = state.players.get_mut(player_id).unwrap();
            p.is_ready = *ready;

            // Check Consensus
            let all_ready = state.players.values().all(|p| p.is_ready);
            if all_ready {
                state = advance_phase(state)?;
            }
        }
        Action::Pass => {
            is_immediate = true;
            let p = state.players.get_mut(player_id).unwrap();
            p.ap = 0;
            p.is_ready = true;

            // Check Consensus (Same as VoteReady)
            let all_ready = state.players.values().all(|p| p.is_ready);
            if all_ready {
                state = advance_phase(state)?;
            }
        }
        Action::Undo { action_id } => {
            is_immediate = true;
            // 1. Find index
            let idx = state.proposal_queue.iter().position(|p| p.id == *action_id);

            if let Some(i) = idx {
                let proposal = &state.proposal_queue[i];

                // 2. Verify Owner
                if proposal.player_id != player_id {
                    return Err(GameError::InvalidAction(
                        "Cannot undo another player's action".to_string(),
                    ));
                }

                // 3. Remove
                let removed = state.proposal_queue.remove(i);

                // 4. Refund AP
                let refund = action_cost(&state, &removed.action);
                if let Some(p) = state.players.get_mut(player_id) {
                    p.ap += refund;
                }
            } else {
                return Err(GameError::InvalidAction(
                    "Action not found to undo".to_string(),
                ));
            }
        }
        Action::Move { to_room } => {
            // Pathfinding Logic
            let pid_string = player_id.to_string();
            let start_room = get_player_projected_room(&state, &pid_string);

            if let Some(path) = find_path(&state.map, start_room, *to_room) {
                let step_cost = base_cost;
                let total_cost = step_cost * (path.len() as i32);

                if state.players.get(player_id).unwrap().ap < total_cost {
                    return Err(GameError::NotEnoughAP);
                }

                // Queue all steps
                for step_room in path {
                    state.proposal_queue.push(ProposedAction {
                        id: Uuid::new_v4().to_string(),
                        player_id: player_id.to_string(),
                        action: Action::Move { to_room: step_room },
                    });
                }

                // Deduct AP
                let p = state.players.get_mut(player_id).unwrap();
                p.ap -= total_cost;
                ap_deducted = true;
            } else {
                return Err(GameError::InvalidMove);
            }
        }
        // Queued Actions (Other than Move)
        Action::Extinguish => {
            // Simulate queue to check if fire will still exist
            let mut room_fire_counts: std::collections::HashMap<u32, usize> = state
                .map
                .rooms
                .iter()
                .map(|(id, r)| {
                    (
                        *id,
                        r.hazards.iter().filter(|&&h| h == HazardType::Fire).count(),
                    )
                })
                .collect();

            let mut player_positions: std::collections::HashMap<String, u32> = state
                .players
                .iter()
                .map(|(id, p)| (id.clone(), p.room_id))
                .collect();

            for prop in &state.proposal_queue {
                if let Action::Move { to_room } = &prop.action {
                    if let Some(pos) = player_positions.get_mut(&prop.player_id) {
                        *pos = *to_room;
                    }
                }
                if let Action::Extinguish = &prop.action {
                    if let Some(pos) = player_positions.get(&prop.player_id) {
                        if let Some(count) = room_fire_counts.get_mut(pos) {
                            if *count > 0 {
                                *count -= 1;
                            }
                        }
                    }
                }
            }

            let my_pos = player_positions
                .get(player_id)
                .ok_or(GameError::PlayerNotFound)?;
            let fires_left = room_fire_counts.get(my_pos).unwrap_or(&0);

            if *fires_left == 0 {
                return Err(GameError::InvalidAction(
                    "No fire to extinguish (or already targeted)".to_string(),
                ));
            }

            state.proposal_queue.push(ProposedAction {
                id: Uuid::new_v4().to_string(),
                player_id: player_id.to_string(),
                action: action.clone(),
            });
        }
        // Queued Actions (Everything else)
        _ => {
            // DEFERRED VALIDATION:
            // We do NOT check neighbors, systems, or inventory here.
            // This allows queuing "Move to Hallway" -> "Move to Kitchen" even if currently in Engine.
            // Validation happens in `resolution::resolve_proposal_queue`.

            // Queue it
            state.proposal_queue.push(ProposedAction {
                id: Uuid::new_v4().to_string(),
                player_id: player_id.to_string(),
                action: action.clone(),
            });
        }
    }

    // 3. Deduct AP (Paid immediately)
    if (!is_immediate && !ap_deducted) || matches!(action, Action::Pass) {
        // Re-borrow because VoteReady/Pass modified player or state passed to advance_phase might have consumed it?
        // Actually `advance_phase` takes ownership of `state`.
        // If `VoteReady` triggered advance_phase, `state` is already new state.
        // Wait, `VoteReady` logic above: `state = advance_phase(state)?;`
        // So if we advanced phase, we are fine.

        // Only deduct AP if we didn't just advance phase/Pass?
        // Pass sets AP=0 manually.
        // VoteReady costs 0.
        // Chat costs 0.
        // So we only deduct for Queued actions.

        if !is_immediate {
            let player = state.players.get_mut(player_id).unwrap();
            player.ap -= base_cost;
        }
    }

    // 4. Update Sequence
    state.sequence_id += 1;

    Ok(state)
}

fn advance_phase(mut state: GameState) -> Result<GameState, GameError> {
    match state.phase {
        GamePhase::Lobby => {
            state.phase = GamePhase::MorningReport;
            state.shields_active = false;
            state.evasion_active = false;
            cards::draw_card(&mut state);
            for p in state.players.values_mut() {
                p.is_ready = false;
            }
        }
        GamePhase::MorningReport => {
            state.phase = GamePhase::EnemyTelegraph;

            // Archive the event
            state.latest_event = None;

            // Generate telegraph
            let mut rng = StdRng::seed_from_u64(state.rng_seed);
            let target_room = rng.gen_range(2..=11);
            state.rng_seed = rng.gen();

            state.enemy.next_attack = Some(EnemyAttack {
                target_room,
                effect: AttackEffect::Fireball,
            });

            // Reset ready
            for p in state.players.values_mut() {
                p.is_ready = false;
            }
        }
        GamePhase::EnemyTelegraph => {
            state.phase = GamePhase::TacticalPlanning;
            // Reset AP
            for p in state.players.values_mut() {
                p.ap = 2;
                p.is_ready = false;
            }
        }
        GamePhase::TacticalPlanning => {
            state.phase = GamePhase::Execution;
            for p in state.players.values_mut() {
                p.is_ready = false;
            }

            // RESOLVE ACTIONS
            resolution::resolve_proposal_queue(&mut state);
        }
        GamePhase::Execution => {
            // Check if any player still has AP
            let any_ap_left = state.players.values().any(|p| p.ap > 0);

            if any_ap_left {
                // Back to Planning
                state.phase = GamePhase::TacticalPlanning;
                for p in state.players.values_mut() {
                    p.is_ready = false;
                }
            } else {
                // Proceed to End of Round
                state.phase = GamePhase::EnemyAction;
                // Run Logic
                resolution::resolve_enemy_attack(&mut state);
                resolution::resolve_hazards(&mut state);

                for p in state.players.values_mut() {
                    p.is_ready = false;
                }
            }
        }
        GamePhase::EnemyAction => {
            state.turn_count += 1;
            state.phase = GamePhase::MorningReport;
            state.shields_active = false;
            state.evasion_active = false;

            // Respawn Logic
            for p in state.players.values_mut() {
                if p.status.contains(&PlayerStatus::Fainted) {
                    p.status.retain(|s| *s != PlayerStatus::Fainted);
                    p.hp = 3;
                    p.room_id = 3; // Dormitory
                }
            }

            cards::draw_card(&mut state);
            for p in state.players.values_mut() {
                p.is_ready = false;
            }
        }
        _ => {}
    }
    Ok(state)
}

fn action_cost(state: &GameState, action: &Action) -> i32 {
    // C04: Slippery Deck check
    let slippery = state.active_situations.iter().any(|c| c.id == "C04");

    let base = match action {
        Action::Chat { .. } | Action::VoteReady { .. } => 0,
        Action::Move { .. } => {
            if slippery {
                0
            } else {
                1
            }
        }
        Action::Interact => 1,
        Action::Bake | Action::Shoot | Action::Extinguish | Action::Repair => 1,
        Action::Throw { .. } | Action::PickUp { .. } => 1,
        Action::Revive { .. } => 1,
        Action::RaiseShields | Action::EvasiveManeuvers => 2,
        Action::Drop { .. } => 0,
        Action::Pass => 0,
        Action::Undo { .. } => 0,
        Action::Join { .. } => 0,
        Action::SetName { .. } => 0,
        Action::FullSync { .. } => 0,
    };

    if slippery && base > 0 && !matches!(action, Action::Move { .. }) {
        return base + 1;
    }

    base
}
