use crate::types::*;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Player not found")]
    PlayerNotFound,
    #[error("Room not found")]
    RoomNotFound,
    #[error("Not enough AP")]
    NotEnoughAP,
    #[error("Invalid move: No door")]
    InvalidMove,
    #[error("Room is blocked by hazard")]
    RoomBlocked,
    #[error("Cannot act during silence")]
    Silenced,
    #[error("Invalid item index")]
    InvalidItem,
    #[error("Invalid Action: {0}")]
    InvalidAction(String),
}

pub struct GameLogic;

impl GameLogic {
    pub fn new_game(player_ids: Vec<String>, seed: u64) -> GameState {
        let mut rooms = HashMap::new();
        
        // Define Rooms based on Rules
        // Central Hallway (7) connects to everything? 
        // Rules say: "Via the Hallway (2 AP): From a Room -> Hallway -> Other Room"
        // And "Central Hallway: The Crossroads".
        // Let's assume a star topology for now: 7 is neighbor to all.
        // Or strictly following the board layout if we had it.
        // docs/rules.md says "Central Hallway: The Crossroads".
        // Let's implement a simple star topology centered on 7 for v1.
        
        let room_defs = vec![
            (2, "The Bow", Some(SystemType::Bow)),
            (3, "Dormitory", Some(SystemType::Dormitory)),
            (4, "Cargo", Some(SystemType::Cargo)),
            (5, "Engine", Some(SystemType::Engine)),
            (6, "Kitchen", Some(SystemType::Kitchen)),
            (7, "Central Hallway", Some(SystemType::Hallway)),
            (8, "Cannons", Some(SystemType::Cannons)),
            (9, "Bridge", Some(SystemType::Bridge)),
            (10, "Sickbay", Some(SystemType::Sickbay)),
            (11, "Storage", Some(SystemType::Storage)),
        ];

        for (id, name, sys) in room_defs {
            let mut neighbors = vec![];
            if id != 7 {
                neighbors.push(7);
            } else {
                // Hallway connects to everything else
                neighbors = vec![2, 3, 4, 5, 6, 8, 9, 10, 11];
            }

            // Special items
            let items = if id == 11 {
                // Storage starts with 5 Peppernuts
                vec![ItemType::Peppernut; 5]
            } else {
                vec![]
            };

            rooms.insert(id, Room {
                id,
                name: name.to_string(),
                system: sys,
                hazards: vec![],
                items,
                neighbors,
            });
        }

        let mut players = HashMap::new();
        for (i, pid) in player_ids.into_iter().enumerate() {
            players.insert(pid.clone(), Player {
                id: pid.clone(),
                name: format!("Player {}", i + 1),
                room_id: 3, // Start in Dormitory? Rules say "choose starting pos except Room 1". Let's default to Dormitory.
                hp: 3,
                ap: 2,
                inventory: vec![],
                status: vec![],
                is_ready: false,
            });
        }

        GameState {
            sequence_id: 0,
            rng_seed: seed,
            phase: GamePhase::MorningReport,
            turn_count: 1,
            hull_integrity: 20,
            map: GameMap { rooms },
            players,
            enemy: Enemy {
                name: "The Petty Thief".to_string(),
                hp: 5,
                max_hp: 5,
                next_attack: None,
            },
            chat_log: vec![],
            proposal_queue: vec![],
            active_situations: vec![],
        }
    }

    /// The Core Reducer: Applies a single action to the state.
    /// Returns the new state or an error.
    /// 
    /// If `hypothetical_seed` is provided, it uses that for RNG checks.
    /// If it encounters a necessary RNG check but is in hypothetical mode, it *may* abort (if we implement that pattern),
    /// or just use the seed to give a result.
    pub fn apply_action(
        mut state: GameState, 
        player_id: &str, 
        action: Action, 
        _hypothetical_seed: Option<u64>
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

        // 2. Validate AP (unless it's free)
        let cost = action_cost(&action);
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
                
                // Check hazards in source/dest? (e.g. sugar/glue)
                // For now, basic move.
                player.room_id = to_room;
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
                     // TODO: Use the seed
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
            },
            
            Action::RaiseShields => {
                // Costs 2 AP. 
                // Checks if Bridge.
                // TODO: Set flag in state.
            },
            
            Action::EvasiveManeuvers => {
                // Costs 2 AP.
                // Checks if Engine.
                // TODO: Set flag in state.
            },
            
            Action::Revive { .. } => {
                // Check location + status.
            },
            
            Action::Chat { message } => {
                state.chat_log.push(ChatMessage {
                    sender: player_id.to_string(),
                    text: message,
                    timestamp: 0, // TODO: Use real timestamp or sequence
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
}

fn action_cost(action: &Action) -> i32 {
    match action {
        Action::Chat { .. } | Action::VoteReady { .. } => 0,
        Action::Move { .. } => 1,
        Action::Interact => 1, 
        Action::Bake | Action::Shoot | Action::Extinguish | Action::Repair => 1,
        Action::Throw { .. } | Action::PickUp { .. } => 1,
        Action::Revive { .. } => 1,
        Action::RaiseShields | Action::EvasiveManeuvers => 2,
        Action::Drop { .. } => 0, 
        Action::Pass => 0,
        Action::Join { .. } => 0,
        Action::FullSync { .. } => 0,
    }
}
