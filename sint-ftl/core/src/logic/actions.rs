use super::{
    cards::{self, get_behavior},
    pathfinding::find_path,
    resolution,
};
use crate::logic::handlers::get_handler;
use crate::{logic::GameError, types::*};
use rand::{rngs::StdRng, Rng, SeedableRng};
use uuid::Uuid;

fn deterministic_uuid(state: &mut GameState) -> String {
    let mut rng = StdRng::seed_from_u64(state.rng_seed);
    let mut bytes = [0u8; 16];
    rng.fill(&mut bytes);
    state.rng_seed = rng.gen();

    // Set UUID v4 bits manually to ensure it looks valid
    bytes[6] = (bytes[6] & 0x0f) | 0x40; // Version 4
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // Variant 1

    Uuid::from_bytes(bytes).to_string()
}

pub fn apply_action(
    state: GameState,
    player_id: &str,
    action: Action,
) -> Result<GameState, GameError> {
    match action {
        Action::Meta(meta_action) => apply_meta_action(state, player_id, meta_action),
        Action::Game(game_action) => apply_game_action(state, player_id, game_action),
    }
}

use crate::logic::find_room_with_system_in_map;

fn apply_meta_action(
    mut state: GameState,
    player_id: &str,
    action: MetaAction,
) -> Result<GameState, GameError> {
    match action {
        MetaAction::Join { name } => {
            if state.players.contains_key(player_id) {
                return Ok(state);
            }
            if state.players.values().any(|p| p.name == name) {
                return Err(GameError::InvalidAction("Name already taken".to_owned()));
            }

            // Correctly find the Dormitory's Room ID
            let start_room =
                find_room_with_system_in_map(&state.map, SystemType::Dormitory).unwrap_or(0); // Fallback to 0 if not found

            state.players.insert(
                player_id.to_owned(),
                Player {
                    id: player_id.to_owned(),
                    name,
                    room_id: start_room,
                    hp: 3,
                    ap: 2,
                    inventory: vec![],
                    status: vec![],
                    is_ready: false,
                },
            );
            state.sequence_id += 1;
            Ok(state)
        }
        MetaAction::FullSync { state_json } => {
            match serde_json::from_str::<GameState>(&state_json) {
                Ok(new_state) => Ok(new_state),
                Err(e) => Err(GameError::InvalidAction(format!("Bad Sync: {}", e))),
            }
        }
        MetaAction::SetName { name } => {
            if state.phase != GamePhase::Lobby {
                return Err(GameError::InvalidAction(
                    "Cannot change name after game start".to_owned(),
                ));
            }
            if state
                .players
                .values()
                .any(|p| p.name == name && p.id != player_id)
            {
                return Err(GameError::InvalidAction("Name already taken".to_owned()));
            }
            if let Some(p) = state.players.get_mut(player_id) {
                p.name = name;
            } else {
                return Err(GameError::PlayerNotFound);
            }
            state.sequence_id += 1;
            Ok(state)
        }
        MetaAction::SetMapLayout { layout } => {
            if state.phase != GamePhase::Lobby {
                return Err(GameError::InvalidAction(
                    "Cannot change map layout after game start".to_owned(),
                ));
            }

            state.layout = layout;
            state.map = crate::logic::map_gen::generate_map(layout);

            // Move all players to the new Dormitory
            let start_room =
                find_room_with_system_in_map(&state.map, SystemType::Dormitory).unwrap_or(0);

            for p in state.players.values_mut() {
                p.room_id = start_room;
                // Reset ready status so players must re-confirm
                p.is_ready = false;
            }

            state.sequence_id += 1;
            Ok(state)
        }
    }
}

fn apply_game_action(
    mut state: GameState,
    player_id: &str,
    action: GameAction,
) -> Result<GameState, GameError> {
    // Player Validation: Ensure the player exists for most actions.
    // We check this early to return a clear PlayerNotFound error.
    match &action {
        GameAction::Chat { .. } | GameAction::VoteReady { .. } => {
            // These actions might be used by spectators or have different validation.
            // For now, we still require a valid player ID.
            if !state.players.contains_key(player_id) {
                return Err(GameError::PlayerNotFound);
            }
        }
        _ => {
            if !state.players.contains_key(player_id) {
                return Err(GameError::PlayerNotFound);
            }
        }
    }

    // Phase Restriction: Gameplay actions only in TacticalPlanning
    if state.phase != GamePhase::TacticalPlanning {
        match action {
            GameAction::Chat { .. } | GameAction::VoteReady { .. } => {}
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

    // CRITICAL FIX: Restore proposal_queue to projected_state so cost calculations
    // (like Sugar Rush counting moves) can see what has already been proposed.
    projected_state.proposal_queue = state.proposal_queue.clone();

    // 0. Handle Immediate Actions (Bypass Projection)
    match &action {
        GameAction::Chat { message } => {
            // CARD VALIDATION (Immediate)
            for card in &state.active_situations {
                get_behavior(card.id).validate_action(&state, player_id, &action)?;
            }

            state.chat_log.push(ChatMessage {
                sender: player_id.to_owned(),
                text: message.clone(),
                timestamp: 0,
            });
            state.sequence_id += 1;
            return Ok(state);
        }
        GameAction::VoteReady { ready } => {
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
        GameAction::Pass => {
            for card in &state.active_situations {
                get_behavior(card.id).validate_action(&state, player_id, &action)?;
            }
            let p = state
                .players
                .get_mut(player_id)
                .ok_or(GameError::PlayerNotFound)?;

            if p.ap == 0 {
                return Err(GameError::InvalidAction(
                    "Cannot Pass with 0 AP. Vote ready instead.".to_owned(),
                ));
            }

            p.ap = 0;
            p.is_ready = true;
            if state.players.values().all(|p| p.is_ready) {
                state = advance_phase(state)?;
            }
            state.sequence_id += 1;
            return Ok(state);
        }
        GameAction::Undo { action_id } => {
            let idx = state.proposal_queue.iter().position(|p| p.id == *action_id);
            if let Some(i) = idx {
                let proposal = &state.proposal_queue[i];
                if proposal.player_id != player_id {
                    return Err(GameError::InvalidAction(
                        "Cannot undo another player's action".to_owned(),
                    ));
                }
                let removed = state.proposal_queue.remove(i);
                let refund = action_cost(&state, player_id, &removed.action);
                if let Some(p) = state.players.get_mut(player_id) {
                    p.ap += refund;
                }
            } else {
                return Err(GameError::InvalidAction(
                    "Action not found to undo".to_owned(),
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
        GameAction::Move { to_room } => {
            let start_room = p_proj.room_id;
            if let Some(path) = find_path(&state.map, start_room, *to_room) {
                let step_cost = base_cost; // Per step
                let total_cost = step_cost * (path.len() as i32);

                if current_ap < total_cost {
                    return Err(GameError::NotEnoughAP);
                }

                for step_room in path {
                    let id = deterministic_uuid(&mut state);
                    state.proposal_queue.push(ProposedAction {
                        id,
                        player_id: player_id.to_owned(),
                        action: GameAction::Move { to_room: step_room },
                    });
                }
                let p = state
                    .players
                    .get_mut(player_id)
                    .ok_or(GameError::PlayerNotFound)?;
                p.ap -= total_cost;
            } else {
                return Err(GameError::InvalidMove);
            }
        }
        _ => {
            // Generic Handler Validation
            let handler = get_handler(&action);
            handler.validate(&projected_state, player_id)?;

            // Queue
            let id = deterministic_uuid(&mut state);
            state.proposal_queue.push(ProposedAction {
                id,
                player_id: player_id.to_owned(),
                action: action.clone(),
            });
            let p = state
                .players
                .get_mut(player_id)
                .ok_or(GameError::PlayerNotFound)?;
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

            // Reset AP (Start of Game)
            for p in state.players.values_mut() {
                p.ap = 2;
            }

            cards::draw_card(&mut state);

            // Round Start Hook
            let active_ids: Vec<CardId> = state.active_situations.iter().map(|c| c.id).collect();
            for id in active_ids {
                get_behavior(id).on_round_start(&mut state);
            }

            for p in state.players.values_mut() {
                p.is_ready = false;
            }
        }
        GamePhase::MorningReport => {
            state.phase = GamePhase::EnemyTelegraph;

            // Archive the event
            state.latest_event = None;

            if state.is_resting {
                state.chat_log.push(ChatMessage {
                    sender: "SYSTEM".to_owned(),
                    text: "Rest Round: The horizon is clear.".to_owned(),
                    timestamp: 0,
                });
                state.enemy.next_attack = None;
            } else {
                // Generate telegraph normally
                let mut rng = StdRng::seed_from_u64(state.rng_seed);
                // 2d6 distribution (2-12). 11-12 is a miss.
                let roll = rng.gen_range(1..=6) + rng.gen_range(1..=6);
                state.rng_seed = rng.gen();

                let mut attack = if let Some(sys) = SystemType::from_u32(roll) {
                    let room_id = find_room_with_system_in_map(&state.map, sys).unwrap_or(0);
                    EnemyAttack {
                        target_room: room_id,
                        target_system: Some(sys),
                        effect: AttackEffect::Fireball,
                    }
                } else {
                    EnemyAttack {
                        target_room: 0,
                        target_system: None,
                        effect: AttackEffect::Miss,
                    }
                };

                // Allow cards to modify it (e.g. FogBank masking it)
                let active_ids: Vec<CardId> =
                    state.active_situations.iter().map(|c| c.id).collect();
                for id in active_ids {
                    get_behavior(id).modify_telegraph(&mut attack);
                }

                state.enemy.next_attack = Some(attack);
            }

            // Reset ready
            for p in state.players.values_mut() {
                p.is_ready = false;
            }
        }
        GamePhase::EnemyTelegraph => {
            state.phase = GamePhase::TacticalPlanning;
            // Reset Ready (AP already reset in MorningReport)
            for p in state.players.values_mut() {
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

            // Check Rest Round Logic
            if state.enemy.state == EnemyState::Defeated {
                if state.is_resting {
                    // Rest is over
                    state.is_resting = false;
                    state.boss_level += 1;
                    state.enemy = crate::logic::get_boss(state.boss_level);
                    state.chat_log.push(ChatMessage {
                        sender: "SYSTEM".to_owned(),
                        text: format!("Rest Over! Approaching: {}", state.enemy.name),
                        timestamp: 0,
                    });
                } else {
                    // Start Rest
                    state.is_resting = true;
                    state.chat_log.push(ChatMessage {
                        sender: "SYSTEM".to_owned(),
                        text: "Victory! Taking a rest round...".to_owned(),
                        timestamp: 0,
                    });
                }
            }

            // Reset AP (New Round)
            for p in state.players.values_mut() {
                p.ap = 2;
            }

            if !state.is_resting {
                // Trigger Card End-of-Round Effects (e.g. Timebombs, Mice)
                let active_ids: Vec<CardId> =
                    state.active_situations.iter().map(|c| c.id).collect();
                for id in active_ids {
                    get_behavior(id).on_round_end(&mut state);
                }

                cards::draw_card(&mut state);

                // Round Start Hook (New Round)
                let active_ids_new: Vec<CardId> =
                    state.active_situations.iter().map(|c| c.id).collect();
                for id in active_ids_new {
                    get_behavior(id).on_round_start(&mut state);
                }
            }

            // Respawn Logic
            let dormitory_id =
                find_room_with_system_in_map(&state.map, SystemType::Dormitory).unwrap_or(0);
            for p in state.players.values_mut() {
                if p.status.contains(&PlayerStatus::Fainted) {
                    p.status.retain(|s| *s != PlayerStatus::Fainted);
                    p.hp = 3;
                    p.room_id = dormitory_id;
                }
            }

            for p in state.players.values_mut() {
                p.is_ready = false;
            }
        }
        _ => {}
    }
    Ok(state)
}

pub fn action_cost(state: &GameState, player_id: &str, action: &GameAction) -> i32 {
    // Immediate actions that don't have handlers or have special logic in handlers
    let base = match action {
        GameAction::Chat { .. }
        | GameAction::VoteReady { .. }
        | GameAction::Pass
        | GameAction::Undo { .. } => 0,
        _ => get_handler(action).base_cost(),
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
    actions.push(Action::Game(GameAction::Chat {
        message: "".to_owned(),
    }));

    // Phase-specific checks
    match projected_state.phase {
        GamePhase::Lobby => {
            actions.push(Action::Game(GameAction::VoteReady { ready: true }));
            actions.push(Action::Game(GameAction::VoteReady { ready: false }));
            actions.push(Action::Meta(MetaAction::SetName {
                name: "".to_owned(),
            }));
            return actions;
        }
        GamePhase::MorningReport
        | GamePhase::EnemyTelegraph
        | GamePhase::Execution
        | GamePhase::EnemyAction => {
            actions.push(Action::Game(GameAction::VoteReady { ready: true }));
            return actions;
        }
        GamePhase::TacticalPlanning => {
            actions.push(Action::Game(GameAction::VoteReady { ready: true }));
            actions.push(Action::Game(GameAction::Pass));
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
            let action = GameAction::Move { to_room: neighbor };
            if p.ap >= action_cost(&projected_state, player_id, &action) {
                actions.push(Action::Game(action));
            }
        }

        // System Actions
        let room_functional =
            !room.hazards.contains(&HazardType::Fire) && !room.hazards.contains(&HazardType::Water);

        if room_functional {
            if let Some(sys) = room.system {
                let action = match sys {
                    SystemType::Kitchen => Some(GameAction::Bake),
                    SystemType::Cannons => {
                        if p.inventory.contains(&ItemType::Peppernut) {
                            Some(GameAction::Shoot)
                        } else {
                            None
                        }
                    }
                    SystemType::Engine => Some(GameAction::RaiseShields),
                    SystemType::Bridge => Some(GameAction::EvasiveManeuvers),
                    SystemType::Bow => Some(GameAction::Lookout),
                    _ => None,
                };

                if let Some(act) = action {
                    if p.ap >= action_cost(&projected_state, player_id, &act) {
                        actions.push(Action::Game(act));
                    }
                }

                // Sickbay (Targeted Action)
                if sys == SystemType::Sickbay
                    && p.ap
                        >= action_cost(
                            &projected_state,
                            player_id,
                            &GameAction::FirstAid {
                                target_player: "".to_owned(),
                            },
                        )
                {
                    // Target Self
                    actions.push(Action::Game(GameAction::FirstAid {
                        target_player: player_id.to_owned(),
                    }));

                    // Target Neighbors
                    for other_p in projected_state.players.values() {
                        if other_p.id == *player_id {
                            continue;
                        }

                        // Check if in neighbor or same room
                        if room.neighbors.contains(&other_p.room_id) || other_p.room_id == p.room_id
                        {
                            actions.push(Action::Game(GameAction::FirstAid {
                                target_player: other_p.id.clone(),
                            }));
                        }
                    }
                }
            }
        }

        // Hazards
        if room.hazards.contains(&HazardType::Fire) {
            let action = GameAction::Extinguish;
            if p.ap >= action_cost(&projected_state, player_id, &action) {
                actions.push(Action::Game(action));
            }
        }
        if room.hazards.contains(&HazardType::Water)
            || (room.system == Some(SystemType::Cargo) && projected_state.hull_integrity < MAX_HULL)
        {
            let action = GameAction::Repair;
            if p.ap >= action_cost(&projected_state, player_id, &action) {
                actions.push(Action::Game(action));
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
                    let action = GameAction::PickUp {
                        item_type: item.clone(),
                    };
                    if p.ap >= action_cost(&projected_state, player_id, &action) {
                        actions.push(Action::Game(action));
                    }
                }
                seen_items.push(item.clone());
            }
        }

        // Interact (Situation Solutions)
        for card in &projected_state.active_situations {
            if let Some(sol) = &card.solution {
                let room_match = match sol.target_system {
                    Some(sys) => {
                        crate::logic::find_room_with_system_in_map(&projected_state.map, sys)
                            == Some(p.room_id)
                    }
                    None => true, // Solvable anywhere
                };

                let item_match = sol.item_cost.is_none()
                    || p.inventory.contains(sol.item_cost.as_ref().unwrap());

                if room_match && item_match {
                    let action = GameAction::Interact;
                    if p.ap >= action_cost(&projected_state, player_id, &action) {
                        // Avoid duplicates if multiple cards solvable in same room
                        if !actions.contains(&Action::Game(GameAction::Interact)) {
                            actions.push(Action::Game(action));
                        }
                    }
                }
            }
        }

        // Revive
        for other_p in projected_state.players.values() {
            if other_p.id != *player_id
                && other_p.room_id == p.room_id
                && other_p.status.contains(&PlayerStatus::Fainted)
            {
                let action = GameAction::Revive {
                    target_player: other_p.id.clone(),
                };
                if p.ap >= action_cost(&projected_state, player_id, &action) {
                    actions.push(Action::Game(action));
                }
            }
        }
    }

    // Undo (Always valid if actions exist in original state queue)
    for prop in &state.proposal_queue {
        if prop.player_id == player_id {
            actions.push(Action::Game(GameAction::Undo {
                action_id: prop.id.clone(),
            }));
        }
    }

    // FINAL VALIDATION FILTER:
    // Ensure all generated actions are actually permitted by active cards AND game rules.
    actions.retain(|action| {
        if let Action::Game(game_action) = action {
            // 1. Check Card Constraints
            for card in &projected_state.active_situations {
                if let Err(_) =
                    get_behavior(card.id).validate_action(&projected_state, player_id, game_action)
                {
                    return false;
                }
            }
            // 2. Check Game Logic Constraints (e.g. InventoryFull)
            // Note: Some actions like Chat/VoteReady don't have handlers or valid logic in get_handler
            // but those are handled by the pattern match in apply_game_action anyway.
            // We only care about Move/Pick/Shoot etc.
            match game_action {
                GameAction::Chat { .. }
                | GameAction::VoteReady { .. }
                | GameAction::Pass
                | GameAction::Undo { .. } => true,
                _ => {
                    let handler = get_handler(game_action);
                    handler.validate(&projected_state, player_id).is_ok()
                }
            }
        } else {
            true
        }
    });

    actions
}
