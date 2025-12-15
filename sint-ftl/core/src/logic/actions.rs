use super::{
    MAX_PLAYER_AP, MAX_PLAYER_HP,
    cards::{self, get_behavior},
    find_room_with_system_in_map,
    pathfinding::find_path,
    resolution,
};
use crate::logic::handlers::get_handler;
use crate::{logic::GameError, types::*};
use rand::{Rng, SeedableRng, rngs::StdRng};
use uuid::Uuid;

fn deterministic_uuid(state: &mut GameState) -> Uuid {
    // Use a deterministic FNV-1a hash to seed the UUID generation.
    // DefaultHasher is not guaranteed to be stable across executions.
    let mut hash = 0xcbf29ce484222325u64;
    let components = [
        state.rng_seed,
        state.proposal_queue.len() as u64,
        state.sequence_id,
    ];
    for &c in &components {
        for byte in c.to_le_bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }

    let mut rng = StdRng::seed_from_u64(hash);
    let mut bytes = [0u8; 16];
    rng.fill(&mut bytes);

    // Set UUID v4 bits manually to ensure it looks valid
    bytes[6] = (bytes[6] & 0x0f) | 0x40; // Version 4
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // Variant 1

    Uuid::from_bytes(bytes)
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

            state.players.insert(Player {
                id: player_id.to_owned(),
                name,
                room_id: start_room,
                hp: MAX_PLAYER_HP,
                ap: MAX_PLAYER_AP,
                inventory: vec![].into(),
                status: vec![].into(),
                is_ready: false,
            });
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
    if let Some(p) = state.players.get(player_id)
        && p.status.contains(&PlayerStatus::Fainted)
    {
        match action {
            GameAction::Chat { .. } | GameAction::VoteReady { .. } => {}
            _ => {
                return Err(GameError::InvalidAction(
                    "You are fainted and cannot act!".to_owned(),
                ));
            }
        }
    }

    if state.phase != GamePhase::TacticalPlanning {
        match action {
            GameAction::Chat { .. } | GameAction::VoteReady { .. } => {}
            _ => {
                return Err(GameError::InvalidAction(format!(
                    "Cannot act during {:?}",
                    state.phase
                )));
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
                    "Cannot Pass with 0 AP. You are already out of actions for this round."
                        .to_owned(),
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

    // 2. Validate AP (REAL state, not projected)
    // We must spend AP we actually have. If projection predicts a refund,
    // we still can't spend it until the refund actually happens in the real state.
    let p_real = state
        .players
        .get(player_id)
        .ok_or(GameError::PlayerNotFound)?;
    let p_proj = projected_state
        .players
        .get(player_id)
        .ok_or(GameError::PlayerNotFound)?;

    let current_ap = p_real.ap;
    let base_cost = action_cost(&projected_state, player_id, &action);

    if current_ap < base_cost {
        return Err(GameError::NotEnoughAP);
    }

    // 3. Queue Logic (using Projected Context)
    match &action {
        GameAction::Move { to_room } => {
            let start_room = p_proj.room_id;

            // Check if any card allows "Leaping" / "Teleporting" directly (1 AP total)
            let mut can_leap = false;
            for card in &projected_state.active_situations {
                if crate::logic::cards::get_behavior(card.id).can_reach(
                    &projected_state,
                    player_id,
                    *to_room,
                ) {
                    can_leap = true;
                    break;
                }
            }

            if can_leap {
                let id = deterministic_uuid(&mut state);
                state.proposal_queue.push(ProposedAction {
                    id,
                    player_id: player_id.to_owned(),
                    action: GameAction::Move { to_room: *to_room },
                });
                let p = state
                    .players
                    .get_mut(player_id)
                    .ok_or(GameError::PlayerNotFound)?;
                p.ap -= base_cost;
            } else if let Some(path) = find_path(&state.map, start_room, *to_room) {
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
    if state.phase == GamePhase::Victory || state.phase == GamePhase::GameOver {
        return Ok(state);
    }
    match state.phase {
        GamePhase::Lobby => {
            state.phase = GamePhase::MorningReport;
            state.shields_active = false;
            state.evasion_active = false;

            // Reset AP (Start of Game)
            for p in state.players.values_mut() {
                p.ap = MAX_PLAYER_AP;
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
                let roll = rng.random_range(1..=6) + rng.random_range(1..=6);
                state.rng_seed = rng.random();

                let mut attack = if let Some(sys) = SystemType::from_u32(roll) {
                    let room_id = find_room_with_system_in_map(&state.map, sys);
                    EnemyAttack {
                        target_room: room_id,
                        target_system: Some(sys),
                        effect: AttackEffect::Fireball,
                    }
                } else {
                    EnemyAttack {
                        target_room: None,
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

            if state.phase == GamePhase::Victory || state.phase == GamePhase::GameOver {
                // Stay in terminal state
            } else if any_ap_left {
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
                    // Start Rest (1 round)
                    state.is_resting = true;
                    state.chat_log.push(ChatMessage {
                        sender: "SYSTEM".to_owned(),
                        text: "Victory! Taking a rest round...".to_owned(),
                        timestamp: 0,
                    });
                }
            }

            // Reset AP (New Round)
            let base_ap = if state.is_resting {
                MAX_PLAYER_AP * 3
            } else {
                MAX_PLAYER_AP
            };
            for p in state.players.values_mut() {
                p.ap = base_ap;
            }

            if !state.is_resting {
                // Trigger Card End-of-Round Effects & Timebombs
                resolution::process_round_end(&mut state);

                // Check for Game Over after card effects
                let crew_wiped = state
                    .players
                    .values()
                    .all(|p| p.status.contains(&PlayerStatus::Fainted));

                if state.hull_integrity <= 0 || crew_wiped {
                    state.phase = GamePhase::GameOver;
                    return Ok(state);
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
                    p.hp = MAX_PLAYER_HP;
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
        GameAction::Interact => {
            // Find which card would be solved and return its AP cost from the solution struct
            if let Some(idx) = crate::logic::cards::find_solvable_card(state, player_id) {
                if let Some(sol) = &state.active_situations[idx].solution {
                    sol.ap_cost as i32
                } else {
                    1
                }
            } else {
                1
            }
        }
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

    // CRITICAL FIX: Restore proposal_queue to projected_state so card validation
    // (like Seasick or Sugar Rush) can see what has already been proposed.
    projected_state.proposal_queue = state.proposal_queue.clone();

    let mut actions = Vec::new();

    // Player MUST exist to perform any action
    let p_opt = projected_state.players.get(player_id);
    if p_opt.is_none() {
        return actions;
    }

    // Always allowed for registered players
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
            // Use ORIGINAL state AP for Pass. If they queued actions, they can't Pass
            // until they either Undo them or resolve them.
            if let Some(p_orig) = state.players.get(player_id)
                && p_orig.ap > 0
            {
                actions.push(Action::Game(GameAction::Pass));
            }
        }
        _ => {}
    }

    let p = match p_opt {
        Some(player) => player,
        None => return actions,
    };

    if p.status.contains(&PlayerStatus::Fainted) {
        return actions;
    }

    // Use REAL AP for availability checks, but projected state for context
    let p_real = state.players.get(player_id).unwrap(); // Existence checked above
    let p_proj = p_opt.unwrap();
    let current_ap = p_real.ap;

    // Use projected position and AP
    if let Some(room) = projected_state.map.rooms.get(&p_proj.room_id) {
        // Move
        let mut target_rooms = Vec::from_iter(room.neighbors.iter().cloned());

        // Check for card-based reachability (Teleport/Leap)
        for rid in projected_state.map.rooms.keys() {
            if target_rooms.contains(&rid) {
                continue;
            }
            for card in &projected_state.active_situations {
                if crate::logic::cards::get_behavior(card.id).can_reach(
                    &projected_state,
                    player_id,
                    rid,
                ) {
                    target_rooms.push(rid);
                    break;
                }
            }
        }

        for to_room in target_rooms {
            let action = GameAction::Move { to_room };
            if current_ap >= action_cost(&projected_state, player_id, &action) {
                actions.push(Action::Game(action));
            }
        }

        // System Actions
        let water_blocked = room.hazards.contains(&HazardType::Water);
        let fire_blocked = room.hazards.contains(&HazardType::Fire);
        let system_broken = room.is_broken;
        let room_functional = !water_blocked && !fire_blocked && !system_broken;

        if room_functional && let Some(sys) = room.system {
            let action = match sys {
                SystemType::Kitchen => Some(GameAction::Bake),
                SystemType::Cannons => {
                    if p_proj.inventory.contains(&ItemType::Peppernut) {
                        Some(GameAction::Shoot)
                    } else {
                        None
                    }
                }
                SystemType::Bridge => {
                    if !projected_state.shields_active {
                        Some(GameAction::RaiseShields)
                    } else {
                        None
                    }
                }
                SystemType::Engine => {
                    if !projected_state.evasion_active {
                        Some(GameAction::EvasiveManeuvers)
                    } else {
                        None
                    }
                }
                SystemType::Bow => Some(GameAction::Lookout),
                _ => None,
            };

            if let Some(act) = action
                && current_ap >= action_cost(&projected_state, player_id, &act)
            {
                actions.push(Action::Game(act));
            }

            // Sickbay (Targeted Action) - MUST BE FUNCTIONAL
            if sys == SystemType::Sickbay
                && current_ap
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
                    if room.neighbors.contains(&other_p.room_id)
                        || other_p.room_id == p_proj.room_id
                    {
                        actions.push(Action::Game(GameAction::FirstAid {
                            target_player: other_p.id.clone(),
                        }));
                    }
                }
            }
        }

        // Hazards
        if room.hazards.contains(&HazardType::Fire) {
            let action = GameAction::Extinguish;
            if current_ap >= action_cost(&projected_state, player_id, &action) {
                actions.push(Action::Game(action));
            }
        }
        if room.hazards.contains(&HazardType::Water)
            || room.system_health < crate::types::SYSTEM_HEALTH
            || (room.system == Some(SystemType::Cargo) && projected_state.hull_integrity < MAX_HULL)
        {
            let action = GameAction::Repair;
            if current_ap >= action_cost(&projected_state, player_id, &action) {
                actions.push(Action::Game(action));
            }
        }

        // Items
        let mut seen_items = Vec::new();
        for item in &room.items {
            if !seen_items.contains(item) {
                if p_proj.can_add_item(*item) {
                    let action = GameAction::PickUp { item_type: *item };
                    if current_ap >= action_cost(&projected_state, player_id, &action) {
                        actions.push(Action::Game(action));
                    }
                }
                seen_items.push(*item);
            }
        }

        // Inventory Actions (Drop/Throw)
        for idx in 0..p_proj.inventory.len() {
            // 1. Drop (Free)
            actions.push(Action::Game(GameAction::Drop { item_index: idx }));

            // 2. Throw (Costs 1 AP) - Only Peppernuts
            if p_proj.inventory[idx] == ItemType::Peppernut {
                let throw_cost = action_cost(
                    &projected_state,
                    player_id,
                    &GameAction::Throw {
                        target_player: "".to_owned(),
                        item_index: idx,
                    },
                );
                if current_ap >= throw_cost {
                    for other_p in projected_state.players.values() {
                        if other_p.id == *player_id {
                            continue;
                        }

                        // Adjacency check for Throw (Same or adjacent room)
                        if room.neighbors.contains(&other_p.room_id)
                            || other_p.room_id == p_proj.room_id
                        {
                            actions.push(Action::Game(GameAction::Throw {
                                target_player: other_p.id.clone(),
                                item_index: idx,
                            }));
                        }
                    }
                }
            }
        }

        // Interact (Situation Solutions)
        if crate::logic::cards::find_solvable_card(&projected_state, player_id).is_some() {
            let action = GameAction::Interact;
            if current_ap >= action_cost(&projected_state, player_id, &action) {
                actions.push(Action::Game(action));
            }
        }

        // Revive
        for other_p in projected_state.players.values() {
            if other_p.id != *player_id
                && other_p.room_id == p_proj.room_id
                && other_p.status.contains(&PlayerStatus::Fainted)
            {
                let action = GameAction::Revive {
                    target_player: other_p.id.clone(),
                };
                if current_ap >= action_cost(&projected_state, player_id, &action) {
                    actions.push(Action::Game(action));
                }
            }
        }
    }

    // Undo (Always valid if actions exist in original state queue)
    for prop in &state.proposal_queue {
        if prop.player_id == player_id {
            actions.push(Action::Game(GameAction::Undo { action_id: prop.id }));
        }
    }

    // FINAL VALIDATION FILTER:
    // Ensure all generated actions are actually permitted by active cards AND game rules.
    actions.retain(|action| {
        if let Action::Game(game_action) = action {
            // 1. Check Card Constraints
            for card in &projected_state.active_situations {
                if get_behavior(card.id)
                    .validate_action(&projected_state, player_id, game_action)
                    .is_err()
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
