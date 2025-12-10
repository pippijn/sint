pub mod actions;
pub mod cards;
pub mod handlers;
pub mod pathfinding;
pub mod resolution;

pub use actions::apply_action;

use crate::types::*;
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize, Clone)]
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
    #[error("Inventory is full")]
    InventoryFull,
    #[error("Invalid Action: {0}")]
    InvalidAction(String),
}

pub const MIN_ROOM_ID: u32 = 2;
pub const MAX_ROOM_ID: u32 = 11;

const ROOM_DEFINITIONS: &[(SystemType, &str)] = &[
    (SystemType::Bow, "The Bow"),
    (SystemType::Dormitory, "Dormitory"),
    (SystemType::Cargo, "Cargo"),
    (SystemType::Engine, "Engine"),
    (SystemType::Kitchen, "Kitchen"),
    (SystemType::Hallway, "Central Hallway"),
    (SystemType::Cannons, "Cannons"),
    (SystemType::Bridge, "Bridge"),
    (SystemType::Sickbay, "Sickbay"),
    (SystemType::Storage, "Storage"),
];

pub struct GameLogic;

impl GameLogic {
    pub fn new_game(player_ids: Vec<String>, seed: u64) -> GameState {
        let mut rooms = HashMap::new();

        for &(sys, name) in ROOM_DEFINITIONS {
            let id = sys.as_u32();
            let mut neighbors = vec![];
            if sys != SystemType::Hallway {
                neighbors.push(SystemType::Hallway.as_u32());
            } else {
                // Hallway connects to everything else
                neighbors = ROOM_DEFINITIONS
                    .iter()
                    .map(|(s, _)| s.as_u32())
                    .filter(|&r_id| r_id != SystemType::Hallway.as_u32())
                    .collect();
            }

            // Special items
            let items = if sys == SystemType::Storage {
                vec![ItemType::Peppernut; 5]
            } else if sys == SystemType::Cargo {
                vec![ItemType::Wheelbarrow]
            } else if sys == SystemType::Engine {
                vec![ItemType::Extinguisher]
            } else {
                vec![]
            };

            rooms.insert(
                id,
                Room {
                    id,
                    name: name.to_string(),
                    system: Some(sys),
                    hazards: vec![],
                    items,
                    neighbors,
                },
            );
        }

        let mut players = HashMap::new();
        for (i, pid) in player_ids.into_iter().enumerate() {
            players.insert(
                pid.clone(),
                Player {
                    id: pid.clone(),
                    name: format!("Player {}", i + 1),
                    room_id: SystemType::Dormitory.as_u32(),
                    hp: 3,
                    ap: 2,
                    inventory: vec![],
                    status: vec![],
                    is_ready: false,
                },
            );
        }

        // Initialize RNG for shuffling
        let mut rng = StdRng::seed_from_u64(seed);
        let next_seed = rng.gen();

        GameState {
            sequence_id: 0,
            rng_seed: next_seed,
            phase: GamePhase::Lobby,
            turn_count: 1,
            hull_integrity: 20,
            boss_level: 0,
            map: GameMap { rooms },
            players,
            enemy: get_boss(0),
            chat_log: vec![],
            shields_active: false,
            evasion_active: false,
            proposal_queue: vec![],
            active_situations: vec![],
            latest_event: None,
            deck: cards::initialize_deck(&mut rng),
            discard: vec![],
        }
    }

    pub fn apply_action(
        state: GameState,
        player_id: &str,
        action: Action,
        _hypothetical_seed: Option<u64>,
    ) -> Result<GameState, GameError> {
        actions::apply_action(state, player_id, action)
    }
}

const BOSS_DEFINITIONS: &[(&str, i32)] = &[
    ("The Petty Thief", 5),
    ("The Monster", 10),
    ("The Armada", 15),
    ("The Kraken", 20),
];

pub const MAX_BOSS_LEVEL: u32 = BOSS_DEFINITIONS.len() as u32;
pub const MAX_PLAYER_HP: i32 = 3;
pub const MAX_PLAYER_AP: i32 = 2;

pub fn get_boss(level: u32) -> Enemy {
    if let Some((name, hp)) = BOSS_DEFINITIONS.get(level as usize) {
        Enemy {
            name: name.to_string(),
            hp: *hp,
            max_hp: *hp,
            next_attack: None,
        }
    } else {
        Enemy {
            name: "Unknown Threat".to_string(),
            hp: 50,
            max_hp: 50,
            next_attack: None,
        }
    }
}
