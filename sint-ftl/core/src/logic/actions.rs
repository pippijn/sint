use super::cards::{self, get_behavior};
use super::pathfinding::find_path;
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
    // 1. Handle Join & FullSync & SetName (Special Cases - Lobby/Sync)
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
                room_id: crate::logic::ROOM_DORMITORY,
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

    // --- ROBUST PROJECTION START ---
    // For validation, we project the state forward by executing the current queue.
    let mut projected_state = state.clone();
    super::resolution::resolve_proposal_queue(&mut projected_state, true);

    // 0. Handle Immediate Actions (Bypass Projection)
    match &action {
        Action::Chat { message } => {
            // CARD VALIDATION (Immediate)
            for card in &state.active_situations {
                get_behavior(card.id).validate_action(&state, player_id, &action)?;
            }

            state.chat_log.push(ChatMessage {
                sender: player_id.to_string(),
                text: message.clone(),
                timestamp: 0,
            });
            state.sequence_id += 1;
            return Ok(state);
        }
        Action::VoteReady { ready } => {
            for card in &state.active_situations {
                get_behavior(card.id).validate_action(&state, player_id, &action)?;
            }
            let p = state
                .players
                .get_mut(player_id)
                .ok_or(GameError::PlayerNotFound)?;
            p.is_ready = *ready;
            if state.players.values().all(|p| p.is_ready) {
                state = advance_phase(state)?;
            }
            state.sequence_id += 1;
            return Ok(state);
        }
        Action::Pass => {
            for card in &state.active_situations {
                get_behavior(card.id).validate_action(&state, player_id, &action)?;
            }
            let p = state
                .players
                .get_mut(player_id)
                .ok_or(GameError::PlayerNotFound)?;
            p.ap = 0;
            p.is_ready = true;
            if state.players.values().all(|p| p.is_ready) {
                state = advance_phase(state)?;
            }
            state.sequence_id += 1;
            return Ok(state);
        }
        Action::Undo { action_id } => {
            let idx = state.proposal_queue.iter().position(|p| p.id == *action_id);
            if let Some(i) = idx {
                let proposal = &state.proposal_queue[i];
                if proposal.player_id != player_id {
                    return Err(GameError::InvalidAction(
                        "Cannot undo another player's action".to_string(),
                    ));
                }
                let removed = state.proposal_queue.remove(i);
                let refund = action_cost(&state, player_id, &removed.action);
                if let Some(p) = state.players.get_mut(player_id) {
                    p.ap += refund;
                }
            } else {
                return Err(GameError::InvalidAction(
                    "Action not found to undo".to_string(),
                ));
            }
            state.sequence_id += 1;
            return Ok(state);
        }
        _ => {} // Fallthrough
    }

    // 1. CARD VALIDATION (Projected)
    for card in &projected_state.active_situations {
        get_behavior(card.id).validate_action(&projected_state, player_id, &action)?;
    }

    // 2. Validate AP (Projected)
    // `projected_state` has remaining AP because it was cloned from `state` (where AP was deducted).
    let p_proj = projected_state
        .players
        .get(player_id)
        .ok_or(GameError::PlayerNotFound)?;
    let current_ap = p_proj.ap;
    let base_cost = action_cost(&projected_state, player_id, &action);

    if current_ap < base_cost {
        return Err(GameError::NotEnoughAP);
    }

    // 3. Queue Logic (using Projected Context)
    match &action {
        Action::Move { to_room } => {
            let start_room = p_proj.room_id;
            if let Some(path) = find_path(&state.map, start_room, *to_room) {
                let step_cost = base_cost;
                let total_cost = step_cost * (path.len() as i32);

                if current_ap < total_cost {
                    return Err(GameError::NotEnoughAP);
                }

                for step_room in path {
                    state.proposal_queue.push(ProposedAction {
                        id: Uuid::new_v4().to_string(),
                        player_id: player_id.to_string(),
                        action: Action::Move { to_room: step_room },
                    });
                }
                let p = state.players.get_mut(player_id).unwrap();
                p.ap -= total_cost;
            } else {
                return Err(GameError::InvalidMove);
            }
        }
        Action::Extinguish => {
            if let Some(room) = projected_state.map.rooms.get(&p_proj.room_id) {
                if !room.hazards.contains(&HazardType::Fire) {
                    return Err(GameError::InvalidAction(
                        "No fire to extinguish (or already targeted)".to_string(),
                    ));
                }
            }
            state.proposal_queue.push(ProposedAction {
                id: Uuid::new_v4().to_string(),
                player_id: player_id.to_string(),
                action: action.clone(),
            });
            let p = state.players.get_mut(player_id).unwrap();
            p.ap -= base_cost;
        }
        Action::Repair => {
            if let Some(room) = projected_state.map.rooms.get(&p_proj.room_id) {
                if !room.hazards.contains(&HazardType::Water) {
                    return Err(GameError::InvalidAction(
                        "No water to repair (or already targeted)".to_string(),
                    ));
                }
            }
            state.proposal_queue.push(ProposedAction {
                id: Uuid::new_v4().to_string(),
                player_id: player_id.to_string(),
                action: action.clone(),
            });
            let p = state.players.get_mut(player_id).unwrap();
            p.ap -= base_cost;
        }
        Action::Bake | Action::Shoot | Action::RaiseShields | Action::EvasiveManeuvers => {
            let room = projected_state
                .map
                .rooms
                .get(&p_proj.room_id)
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
            if !room.hazards.is_empty() {
                return Err(GameError::RoomBlocked);
            }
            state.proposal_queue.push(ProposedAction {
                id: Uuid::new_v4().to_string(),
                player_id: player_id.to_string(),
                action: action.clone(),
            });
            let p = state.players.get_mut(player_id).unwrap();
            p.ap -= base_cost;
        }
        Action::Lookout => {
            let room = projected_state
                .map
                .rooms
                .get(&p_proj.room_id)
                .ok_or(GameError::RoomNotFound)?;

            if room.system != Some(SystemType::Bow) {
                return Err(GameError::InvalidAction(format!(
                    "Lookout requires being in The Bow (2), but you will be in {} ({})",
                    room.name, room.id
                )));
            }
            if !room.hazards.is_empty() {
                return Err(GameError::RoomBlocked);
            }
            state.proposal_queue.push(ProposedAction {
                id: Uuid::new_v4().to_string(),
                player_id: player_id.to_string(),
                action: action.clone(),
            });
            let p = state.players.get_mut(player_id).unwrap();
            p.ap -= base_cost;
        }
        Action::FirstAid { target_player } => {
            let room = projected_state
                .map
                .rooms
                .get(&p_proj.room_id)
                .ok_or(GameError::RoomNotFound)?;

            if room.system != Some(SystemType::Sickbay) {
                return Err(GameError::InvalidAction(format!(
                    "First Aid requires being in Sickbay (10), but you will be in {} ({})",
                    room.name, room.id
                )));
            }
            if !room.hazards.is_empty() {
                return Err(GameError::RoomBlocked);
            }

            // Validate Target Range (Self or Adjacent Room)
            let target = projected_state
                .players
                .get(target_player)
                .ok_or(GameError::PlayerNotFound)?;

            let is_self = target_player == player_id;
            let is_adjacent = room.neighbors.contains(&target.room_id);
            let is_here = target.room_id == p_proj.room_id;

            if !is_self && !is_adjacent && !is_here {
                return Err(GameError::InvalidAction(
                    "Target for First Aid must be self or in adjacent room".to_string(),
                ));
            }

            state.proposal_queue.push(ProposedAction {
                id: Uuid::new_v4().to_string(),
                player_id: player_id.to_string(),
                action: action.clone(),
            });
            let p = state.players.get_mut(player_id).unwrap();
            p.ap -= base_cost;
        }
        Action::PickUp { item_type } => {
            let room = projected_state
                .map
                .rooms
                .get(&p_proj.room_id)
                .ok_or(GameError::RoomNotFound)?;
            if !room.items.contains(item_type) {
                return Err(GameError::InvalidAction(format!(
                    "Item {:?} not in room (or already picked up)",
                    item_type
                )));
            }

            // Inventory Limit Check
            if *item_type == ItemType::Peppernut {
                let nut_count = p_proj
                    .inventory
                    .iter()
                    .filter(|i| **i == ItemType::Peppernut)
                    .count();
                let has_wheelbarrow = p_proj.inventory.contains(&ItemType::Wheelbarrow);
                let limit = if has_wheelbarrow { 5 } else { 1 };

                if nut_count >= limit {
                    return Err(GameError::InventoryFull);
                }
            }

            state.proposal_queue.push(ProposedAction {
                id: Uuid::new_v4().to_string(),
                player_id: player_id.to_string(),
                action: action.clone(),
            });
            let p = state.players.get_mut(player_id).unwrap();
            p.ap -= base_cost;
        }
        Action::Throw {
            target_player,
            item_index,
        } => {
            if *item_index >= p_proj.inventory.len() {
                return Err(GameError::InvalidItem);
            }
            let target = projected_state
                .players
                .get(target_player)
                .ok_or(GameError::PlayerNotFound)?;
            let my_room = p_proj.room_id;
            let target_room = target.room_id;

            let is_adjacent = if my_room == target_room {
                true
            } else if let Some(r) = projected_state.map.rooms.get(&my_room) {
                r.neighbors.contains(&target_room)
            } else {
                false
            };

            if !is_adjacent {
                return Err(GameError::InvalidAction("Target not in range".to_string()));
            }

            state.proposal_queue.push(ProposedAction {
                id: Uuid::new_v4().to_string(),
                player_id: player_id.to_string(),
                action: action.clone(),
            });
            let p = state.players.get_mut(player_id).unwrap();
            p.ap -= base_cost;
        }
        Action::Drop { item_index } => {
            if *item_index >= p_proj.inventory.len() {
                return Err(GameError::InvalidItem);
            }
            state.proposal_queue.push(ProposedAction {
                id: Uuid::new_v4().to_string(),
                player_id: player_id.to_string(),
                action: action.clone(),
            });
            let p = state.players.get_mut(player_id).unwrap();
            p.ap -= base_cost;
        }
        Action::Revive { target_player } => {
            let target = projected_state
                .players
                .get(target_player)
                .ok_or(GameError::PlayerNotFound)?;
            if target.room_id != p_proj.room_id {
                return Err(GameError::InvalidAction(
                    "Target not in same room".to_string(),
                ));
            }
            if !target.status.contains(&PlayerStatus::Fainted) {
                return Err(GameError::InvalidAction(
                    "Target is not Fainted".to_string(),
                ));
            }
            state.proposal_queue.push(ProposedAction {
                id: Uuid::new_v4().to_string(),
                player_id: player_id.to_string(),
                action: action.clone(),
            });
            let p = state.players.get_mut(player_id).unwrap();
            p.ap -= base_cost;
        }

        _ => {
            state.proposal_queue.push(ProposedAction {
                id: Uuid::new_v4().to_string(),
                player_id: player_id.to_string(),
                action: action.clone(),
            });
            let p = state.players.get_mut(player_id).unwrap();
            p.ap -= base_cost;
        }
    }

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
            let target_room = rng.gen_range(crate::logic::MIN_ROOM_ID..=crate::logic::MAX_ROOM_ID);
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
            resolution::resolve_proposal_queue(&mut state, false);
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

                // CHECK GAME OVER CONDITIONS
                let hull_destroyed = state.hull_integrity <= 0;
                let crew_wiped = state
                    .players
                    .values()
                    .all(|p| p.status.contains(&PlayerStatus::Fainted));

                if hull_destroyed || crew_wiped {
                    state.phase = GamePhase::GameOver;
                }

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
            let active_ids_new: Vec<CardId> =
                state.active_situations.iter().map(|c| c.id).collect();
            for id in active_ids_new {
                get_behavior(id).on_round_start(&mut state);
            }

            // Respawn Logic
            for p in state.players.values_mut() {
                if p.status.contains(&PlayerStatus::Fainted) {
                    p.status.retain(|s| *s != PlayerStatus::Fainted);
                    p.hp = 3;
                    p.room_id = crate::logic::ROOM_DORMITORY;
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

pub fn action_cost(state: &GameState, player_id: &str, action: &Action) -> i32 {
    let base = match action {
        Action::Chat { .. } | Action::VoteReady { .. } => 0,
        Action::Move { .. } => 1,
        Action::Interact => 1,
        Action::Bake | Action::Shoot | Action::Extinguish | Action::Repair => 1,
        Action::Lookout | Action::FirstAid { .. } => 1,
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
    // ROBUST IMPLEMENTATION: State Projection
    // We must validate actions against the *projected* state (after the queue executes),
    // not the current state. This allows chaining (e.g., Move -> Shoot).

    let mut projected_state = state.clone();

    // Replay the queue using the official resolution logic to determine
    // the final position, inventory, and status of the player.
    // This correctly updates room_id, inventory, etc. based on the queued actions.
    resolution::resolve_proposal_queue(&mut projected_state, true);

    let mut actions = Vec::new();

    // Always allowed (technically)
    actions.push(Action::Chat {
        message: "".to_string(),
    });

    // Phase-specific checks
    match projected_state.phase {
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

    let p = match projected_state.players.get(player_id) {
        Some(player) => player,
        None => return actions,
    };

    // Use projected position and AP
    if let Some(room) = projected_state.map.rooms.get(&p.room_id) {
        // Move
        for &neighbor in &room.neighbors {
            let action = Action::Move { to_room: neighbor };
            if p.ap >= action_cost(&projected_state, player_id, &action) {
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
                    SystemType::Engine => Some(Action::RaiseShields),
                    SystemType::Bridge => Some(Action::EvasiveManeuvers),
                    SystemType::Bow => Some(Action::Lookout),
                    _ => None,
                };

                if let Some(act) = action {
                    if p.ap >= action_cost(&projected_state, player_id, &act) {
                        actions.push(act);
                    }
                }

                // Sickbay (Targeted Action)
                if sys == SystemType::Sickbay {
                    if p.ap
                        >= action_cost(
                            &projected_state,
                            player_id,
                            &Action::FirstAid {
                                target_player: "".to_string(),
                            },
                        )
                    {
                        // Target Self
                        actions.push(Action::FirstAid {
                            target_player: player_id.to_string(),
                        });

                        // Target Neighbors
                        for other_p in projected_state.players.values() {
                            if other_p.id == *player_id {
                                continue;
                            }

                            // Check if in neighbor or same room
                            if room.neighbors.contains(&other_p.room_id)
                                || other_p.room_id == p.room_id
                            {
                                actions.push(Action::FirstAid {
                                    target_player: other_p.id.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Hazards
        if room.hazards.contains(&HazardType::Fire) {
            let action = Action::Extinguish;
            if p.ap >= action_cost(&projected_state, player_id, &action) {
                actions.push(action);
            }
        }
        if room.hazards.contains(&HazardType::Water) {
            let action = Action::Repair;
            if p.ap >= action_cost(&projected_state, player_id, &action) {
                actions.push(action);
            }
        }

        // Items
        let mut seen_items = Vec::new();
        for item in &room.items {
            if !seen_items.contains(item) {
                let mut can_pickup = true;
                if *item == ItemType::Peppernut {
                    let nut_count = p
                        .inventory
                        .iter()
                        .filter(|i| **i == ItemType::Peppernut)
                        .count();
                    let has_wheelbarrow = p.inventory.contains(&ItemType::Wheelbarrow);
                    let limit = if has_wheelbarrow { 5 } else { 1 };
                    if nut_count >= limit {
                        can_pickup = false;
                    }
                }

                if can_pickup {
                    let action = Action::PickUp {
                        item_type: item.clone(),
                    };
                    if p.ap >= action_cost(&projected_state, player_id, &action) {
                        actions.push(action);
                    }
                }
                seen_items.push(item.clone());
            }
        }
    }

    // Undo (Always valid if actions exist in original state queue)
    for prop in &state.proposal_queue {
        if prop.player_id == player_id {
            actions.push(Action::Undo {
                action_id: prop.id.clone(),
            });
        }
    }

    actions
}
