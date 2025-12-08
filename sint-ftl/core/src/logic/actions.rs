use crate::types::*;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use crate::logic::GameError;
use super::cards;
use super::resolution;

pub fn apply_action(
    mut state: GameState, 
    player_id: &str, 
    action: Action
) -> Result<GameState, GameError> {
    
    // 1. Handle Join & FullSync (Special Cases)
    if let Action::Join { name } = &action {
        if state.players.contains_key(player_id) {
            return Ok(state);
        }
        state.players.insert(player_id.to_string(), Player {
            id: player_id.to_string(),
            name: name.clone(),
            room_id: 3, hp: 3, ap: 2, inventory: vec![], status: vec![], is_ready: false,
        });
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
    
    // Start Game Special Case
    if let Action::StartGame = &action {
        if state.phase != GamePhase::Lobby {
            return Err(GameError::InvalidAction("Game already started".to_string()));
        }
        if state.players.is_empty() {
            return Err(GameError::InvalidAction("No players".to_string()));
        }
        
        state.phase = GamePhase::MorningReport;
        cards::draw_card(&mut state);
        state.sequence_id += 1;
        return Ok(state);
    }

    // Phase Restriction: Gameplay actions only in TacticalPlanning
    if state.phase != GamePhase::TacticalPlanning {
        match action {
            Action::Chat { .. } | Action::VoteReady { .. } => {},
            _ => return Err(GameError::InvalidAction(format!("Cannot act during {:?}", state.phase))),
        }
    }

    // 2. Validate AP (unless it's free)
    let cost = action_cost(&state, &action);
    let player = state.players.get_mut(player_id).ok_or(GameError::PlayerNotFound)?;
    
    if player.ap < cost {
        return Err(GameError::NotEnoughAP);
    }

    // 3. Execute Logic
    match action {
        Action::Move { to_room } => {
            let current_room_id = player.room_id;
            let room = state.map.rooms.get(&current_room_id).ok_or(GameError::RoomNotFound)?;
            
            if !room.neighbors.contains(&to_room) {
                return Err(GameError::InvalidMove);
            }
            
            // C03: Seagull Attack - Cannot move with Peppernut
            let seagull = state.active_situations.iter().any(|c| c.id == "C03");
            if seagull && player.inventory.contains(&ItemType::Peppernut) {
                return Err(GameError::InvalidAction("Seagulls attack! Cannot move with food.".to_string()));
            }
            
            player.room_id = to_room;
        },
        
        Action::Interact => {
            // Check if solving a situation
            let room_id = player.room_id;
            let mut solved_idx = None;
            
            for (i, card) in state.active_situations.iter().enumerate() {
                if let Some(sol) = &card.solution {
                    // Check Room
                    if let Some(req_room) = sol.room_id {
                        if req_room != room_id { continue; }
                    }
                    // Check Item
                    if let Some(req_item) = &sol.item_cost {
                        if !player.inventory.contains(req_item) { continue; }
                    }
                    
                    // Found solvable card
                    solved_idx = Some(i);
                    // Pay Item cost
                    if let Some(req_item) = &sol.item_cost {
                        if let Some(pos) = player.inventory.iter().position(|x| x == req_item) {
                            player.inventory.remove(pos);
                        }
                    }
                    break;
                }
            }
            
            if let Some(idx) = solved_idx {
                state.active_situations.remove(idx);
            } else {
                 return Err(GameError::InvalidAction("Nothing to interact with here".to_string()));
            }
        },
        
        Action::Extinguish => {
            let room = state.map.rooms.get_mut(&player.room_id).ok_or(GameError::RoomNotFound)?;
            if let Some(idx) = room.hazards.iter().position(|&h| h == HazardType::Fire) {
                room.hazards.remove(idx);
            } else {
                return Err(GameError::InvalidAction("No fire here".to_string()));
            }
        },
        
        Action::Bake => {
            let room = state.map.rooms.get_mut(&player.room_id).ok_or(GameError::RoomNotFound)?;
            if room.system != Some(SystemType::Kitchen) {
                return Err(GameError::InvalidAction("Must be in Kitchen to Bake".to_string()));
            }
            // Check if disabled
            if !room.hazards.is_empty() {
                return Err(GameError::RoomBlocked);
            }
            
            // Add peppernuts to room floor
            room.items.push(ItemType::Peppernut);
            room.items.push(ItemType::Peppernut);
            room.items.push(ItemType::Peppernut);
        },
        
        Action::Shoot => {
             let room = state.map.rooms.get_mut(&player.room_id).ok_or(GameError::RoomNotFound)?;
             if room.system != Some(SystemType::Cannons) {
                 return Err(GameError::InvalidAction("Must be in Cannons to Shoot".to_string()));
             }
             
             // Deduct ammo (from player inventory)
             if let Some(idx) = player.inventory.iter().position(|i| *i == ItemType::Peppernut) {
                 player.inventory.remove(idx);
                 
                 // Roll for Hit
                 let mut rng = StdRng::seed_from_u64(state.rng_seed);
                 let roll: u32 = rng.gen_range(1..=6);
                 state.rng_seed = rng.gen(); // Advance seed
                 
                 if roll >= 3 {
                     state.enemy.hp -= 1;
                 }
             } else {
                 // No ammo
             }
        },
        
        Action::Pass => {
            player.ap = 0;
            player.is_ready = true;
        },
        
        Action::VoteReady { ready } => {
            player.is_ready = ready;
            
            // Check Consensus
            let all_ready = state.players.values().all(|p| p.is_ready);
            if all_ready {
                state = advance_phase(state)?;
            }
        },
        
        Action::RaiseShields => {
            // Costs 2 AP. 
        },
        
        Action::EvasiveManeuvers => {
            // Costs 2 AP.
        },
        
        Action::Revive { .. } => {
            // Check location + status.
        },
        
        Action::Chat { message } => {
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
                text: message,
                timestamp: 0, 
            });
        },

        // TODO: Implement other actions
        _ => {}
    }

    // 3. Deduct AP
    // We re-borrow because `player` was mutable above
    let player = state.players.get_mut(player_id).unwrap(); 
    player.ap -= cost;

    // 4. Update Sequence
    state.sequence_id += 1;

    Ok(state)
}

fn advance_phase(mut state: GameState) -> Result<GameState, GameError> {
    match state.phase {
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
            for p in state.players.values_mut() { p.is_ready = false; }
        },
        GamePhase::EnemyTelegraph => {
            state.phase = GamePhase::TacticalPlanning;
            // Reset AP
            for p in state.players.values_mut() {
                p.ap = 2;
                p.is_ready = false;
            }
        },
        GamePhase::TacticalPlanning => {
            state.phase = GamePhase::Execution;
             for p in state.players.values_mut() { p.is_ready = false; }
        },
        GamePhase::Execution => {
            // Check if any player still has AP
            let any_ap_left = state.players.values().any(|p| p.ap > 0);
            
            if any_ap_left {
                // Back to Planning
                state.phase = GamePhase::TacticalPlanning;
                for p in state.players.values_mut() { p.is_ready = false; }
            } else {
                // Proceed to End of Round
                state.phase = GamePhase::EnemyAction;
                // Run Logic
                resolution::resolve_enemy_attack(&mut state);
                resolution::resolve_hazards(&mut state);
                
                for p in state.players.values_mut() { p.is_ready = false; }
            }
        },
        GamePhase::EnemyAction => {
            state.turn_count += 1;
            state.phase = GamePhase::MorningReport;
            
            // Respawn Logic
            for p in state.players.values_mut() {
                if p.status.contains(&PlayerStatus::Fainted) {
                    p.status.retain(|s| *s != PlayerStatus::Fainted);
                    p.hp = 3;
                    p.room_id = 3; // Dormitory
                }
            }
            
            cards::draw_card(&mut state);
            for p in state.players.values_mut() { p.is_ready = false; }
        },
        _ => {}
    }
    Ok(state)
}


fn action_cost(state: &GameState, action: &Action) -> i32 {
    // C04: Slippery Deck check
    let slippery = state.active_situations.iter().any(|c| c.id == "C04");
    
    let base = match action {
        Action::Chat { .. } | Action::VoteReady { .. } => 0,
        Action::Move { .. } => if slippery { 0 } else { 1 },
        Action::Interact => 1, 
        Action::Bake | Action::Shoot | Action::Extinguish | Action::Repair => 1,
        Action::Throw { .. } | Action::PickUp { .. } => 1,
        Action::Revive { .. } => 1,
        Action::RaiseShields | Action::EvasiveManeuvers => 2,
        Action::Drop { .. } => 0, 
        Action::Pass => 0,
        Action::Join { .. } => 0,
        Action::FullSync { .. } => 0,
        Action::StartGame => 0,
    };
    
    if slippery && base > 0 && !matches!(action, Action::Move { .. }) {
        return base + 1;
    }
    
    base
}
