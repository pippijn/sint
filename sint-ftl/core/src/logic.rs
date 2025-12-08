pub mod cards;
pub mod resolution;
pub mod actions;

pub use actions::apply_action;

use crate::types::*;
use std::collections::HashMap;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
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
                room_id: 3, 
                hp: 3,
                ap: 2,
                inventory: vec![],
                status: vec![],
                is_ready: false,
            });
        }

        // Initialize RNG for shuffling
        let mut rng = StdRng::seed_from_u64(seed);
        let next_seed = rng.gen();

        let state = GameState {
            sequence_id: 0,
            rng_seed: next_seed,
            phase: GamePhase::Lobby,
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
            latest_event: None,
            deck: cards::initialize_deck(&mut rng),
            discard: vec![],
        };
        
        state
    }

    pub fn apply_action(
        state: GameState, 
        player_id: &str, 
        action: Action, 
        _hypothetical_seed: Option<u64>
    ) -> Result<GameState, GameError> {
        actions::apply_action(state, player_id, action)
    }
}