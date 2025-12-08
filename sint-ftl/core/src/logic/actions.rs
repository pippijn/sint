use super::cards::{self, get_behavior};
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

    // CARD VALIDATION HOOK
    for card in &state.active_situations {
        get_behavior(card.id).validate_action(&state, player_id, &action)?;
    }

    // 2. Validate AP (unless it's free)
    let base_cost = action_cost(&state, player_id, &action);
    
    // Read-only check for AP
    let current_ap = state.players.get(player_id)
        .ok_or(GameError::PlayerNotFound)?
        .ap;

    if current_ap < base_cost {
        return Err(GameError::NotEnoughAP);
    }

    // 3. Execute OR Queue Logic
    let mut is_immediate = false;
    let mut ap_deducted = false;

    match &action {
        Action::Chat { message } => {
            is_immediate = true;
            // C02: Static Noise handled by CARD VALIDATION HOOK above.
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
                let refund = action_cost(&state, player_id, &removed.action);
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

            // C03 Check moved to registry validation.

            if let Some(path) = find_path(&state.map, start_room, *to_room) {
                let step_cost = base_cost;
                let total_cost = step_cost * (path.len() as i32);

                if current_ap < total_cost {
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
        // Queued Actions (System)
        Action::Bake | Action::Shoot | Action::RaiseShields | Action::EvasiveManeuvers => {
            let pid_string = player_id.to_string();
            let room_id = get_player_projected_room(&state, &pid_string);
            let room = state
                .map
                .rooms
                .get(&room_id)
                .ok_or(GameError::RoomNotFound)?;

            let expected_sys = match action {
                Action::Bake => SystemType::Kitchen,
                Action::Shoot => SystemType::Cannons,
                Action::RaiseShields => SystemType::Engine,
                Action::EvasiveManeuvers => SystemType::Bridge,
                _ => unreachable!(),
            };

            if room.system != Some(expected_sys) {
                return Err(GameError::InvalidAction(format!(
                    "Action {:?} requires being in {:?}, but you will be in {} ({})",
                    action, expected_sys, room.name, room.id
                )));
            }

            // Check Hazards (Room Functionality)
            if !room.hazards.is_empty() {
                return Err(GameError::RoomBlocked);
            }

            state.proposal_queue.push(ProposedAction {
                id: Uuid::new_v4().to_string(),
                player_id: player_id.to_string(),
                action: action.clone(),
            });
        }
        Action::Repair => {
            let pid_string = player_id.to_string();
            let room_id = get_player_projected_room(&state, &pid_string);
            let room = state
                .map
                .rooms
                .get(&room_id)
                .ok_or(GameError::RoomNotFound)?;

            if !room.hazards.contains(&HazardType::Water) {
                return Err(GameError::InvalidAction("No water to repair".to_string()));
            }

            state.proposal_queue.push(ProposedAction {
                id: Uuid::new_v4().to_string(),
                player_id: player_id.to_string(),
                action: action.clone(),
            });
        }
        Action::PickUp { item_type } => {
            let pid_string = player_id.to_string();
            let room_id = get_player_projected_room(&state, &pid_string);
            let room = state
                .map
                .rooms
                .get(&room_id)
                .ok_or(GameError::RoomNotFound)?;

            if !room.items.contains(item_type) {
                // Note: This check is imperfect as it ignores queue consumption,
                // but prevents picking up from empty rooms.
                return Err(GameError::InvalidAction(format!(
                    "Item {:?} not in room",
                    item_type
                )));
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
            
            // Round Start Hook
            let active_ids: Vec<CardId> = state.active_situations.iter().map(|c| c.id).collect();
            for id in active_ids {
                get_behavior(id).on_round_start(&mut state);
            }

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

            // Trigger Card End-of-Round Effects (e.g. Timebombs, Mice)
            let active_ids: Vec<CardId> = state.active_situations.iter().map(|c| c.id).collect();
            for id in active_ids {
                get_behavior(id).on_round_end(&mut state);
            }
            
            // Round Start Hook (New Round)
            let active_ids_new: Vec<CardId> = state.active_situations.iter().map(|c| c.id).collect();
            for id in active_ids_new {
                get_behavior(id).on_round_start(&mut state);
            }

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

fn action_cost(state: &GameState, player_id: &str, action: &Action) -> i32 {
    let base = match action {
        Action::Chat { .. } | Action::VoteReady { .. } => 0,
        Action::Move { .. } => 1,
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

    let mut cost = base;
    for card in &state.active_situations {
        cost = get_behavior(card.id).modify_action_cost(state, player_id, action, cost);
    }
    cost
}

pub fn get_valid_actions(state: &GameState, player_id: &str) -> Vec<Action> {
    let mut actions = Vec::new();

    // Always allowed (technically)
    actions.push(Action::Chat {
        message: "".to_string(),
    }); // Placeholder, client handles text input

    // Phase-specific checks
    match state.phase {
        GamePhase::Lobby => {
            actions.push(Action::VoteReady { ready: true });
            actions.push(Action::VoteReady { ready: false });
            actions.push(Action::SetName {
                name: "".to_string(),
            });
            return actions;
        }
        GamePhase::MorningReport
        | GamePhase::EnemyTelegraph
        | GamePhase::Execution
        | GamePhase::EnemyAction => {
            actions.push(Action::VoteReady { ready: true });
            return actions;
        }
        GamePhase::TacticalPlanning => {
            actions.push(Action::VoteReady { ready: true });
            actions.push(Action::Pass);
        }
        _ => {}
    }

    let p = match state.players.get(player_id) {
        Some(player) => player,
        None => return actions,
    };

    // Calculate projected state/position for validation?
    // For now, valid actions are based on CURRENT state + AP check.
    // Ideally we'd use the projected state from the proposal queue, but that's complex.
    // Let's stick to current state for simplicity, or maybe subtract pending AP?
    // The Client UI currently checks `p.ap`.

    // Move
    let current_room_id = p.room_id; // Or projected?
                                     // Using current room for valid ACTIONS generation.
                                     // The apply_action logic handles pathfinding/queueing.
                                     // But we want to know "Can I click 'Move to Room 5'?"
                                     // For `Move`, we can list all rooms (or just neighbors).
                                     // Since we support pathfinding, we could list ALL rooms, but that's a lot.
                                     // Let's list NEIGHBORS for direct moves, or maybe key rooms?
                                     // Let's list Neighbors for now as "Atomic" actions.

    if let Some(room) = state.map.rooms.get(&current_room_id) {
        // Move
        for &neighbor in &room.neighbors {
            let action = Action::Move { to_room: neighbor };
            if p.ap >= action_cost(state, player_id, &action) {
                actions.push(action);
            }
        }

        // System Actions
        let room_functional =
            !room.hazards.contains(&HazardType::Fire) && !room.hazards.contains(&HazardType::Water);

        if room_functional {
            if let Some(sys) = room.system {
                let action = match sys {
                    SystemType::Kitchen => Some(Action::Bake),
                    SystemType::Cannons => Some(Action::Shoot),
                    SystemType::Engine => Some(Action::RaiseShields), // Swapped
                    SystemType::Bridge => Some(Action::EvasiveManeuvers), // Swapped
                    // Sickbay, etc.
                    _ => None,
                };

                if let Some(act) = action {
                    if p.ap >= action_cost(state, player_id, &act) {
                        actions.push(act);
                    }
                }
            }
        }

        // Hazards
        if room.hazards.contains(&HazardType::Fire) {
            let action = Action::Extinguish;
            if p.ap >= action_cost(state, player_id, &action) {
                actions.push(action);
            }
        }
        if room.hazards.contains(&HazardType::Water) {
            let action = Action::Repair;
            if p.ap >= action_cost(state, player_id, &action) {
                actions.push(action);
            }
        }

        // Items
        // Dedup items
        let mut seen_items = Vec::new();
        for item in &room.items {
            if !seen_items.contains(item) {
                let action = Action::PickUp {
                    item_type: item.clone(),
                };
                if p.ap >= action_cost(state, player_id, &action) {
                    actions.push(action);
                }
                seen_items.push(item.clone());
            }
        }
    }

    // Inventory Actions (Drop/Throw) - Simplified for now

    // Undo
    // Check if player has pending actions
    for prop in &state.proposal_queue {
        if prop.player_id == player_id {
            actions.push(Action::Undo {
                action_id: prop.id.clone(),
            });
        }
    }

    actions
}
