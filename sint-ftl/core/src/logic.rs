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

// Room 0 is the Hub (None).
// Rooms 1-9 are Systems.
const ROOM_DEFINITIONS: &[(Option<SystemType>, &str)] = &[
    (None, "Central Hallway"),              // 0
    (Some(SystemType::Bow), "The Bow"),     // 1
    (Some(SystemType::Dormitory), "Dormitory"), // 2
    (Some(SystemType::Cargo), "Cargo"),     // 3
    (Some(SystemType::Engine), "Engine"),   // 4
    (Some(SystemType::Kitchen), "Kitchen"), // 5
    (Some(SystemType::Cannons), "Cannons"), // 6
    (Some(SystemType::Bridge), "Bridge"),   // 7
    (Some(SystemType::Sickbay), "Sickbay"), // 8
    (Some(SystemType::Storage), "Storage"), // 9
];

pub struct GameLogic;

impl GameLogic {
    pub fn new_game(player_ids: Vec<String>, seed: u64) -> GameState {
        let mut rooms = HashMap::new();
        let hub_id = 0;

        // Star Layout Construction
        for (i, (sys, name)) in ROOM_DEFINITIONS.iter().enumerate() {
            let id = i as u32;
            let mut neighbors = vec![];

            if id == hub_id {
                // Hub connects to all other rooms (1..N)
                for j in 1..ROOM_DEFINITIONS.len() {
                    neighbors.push(j as u32);
                }
            } else {
                // Spoke connects only to Hub
                neighbors.push(hub_id);
            }

            // Special items
            let items = if *sys == Some(SystemType::Storage) {
                vec![ItemType::Peppernut; 5]
            } else if *sys == Some(SystemType::Cargo) {
                vec![ItemType::Wheelbarrow]
            } else if *sys == Some(SystemType::Engine) {
                vec![ItemType::Extinguisher]
            } else {
                vec![]
            };

            rooms.insert(
                id,
                Room {
                    id,
                    name: name.to_string(),
                    system: *sys,
                    hazards: vec![],
                    items,
                    neighbors,
                },
            );
        }

        let map = GameMap { rooms };
        
        // Determine Start Room (Dormitory)
        let start_room = find_room_with_system_in_map(&map, SystemType::Dormitory).unwrap_or(0);

        let mut players = HashMap::new();
        for (i, pid) in player_ids.into_iter().enumerate() {
            players.insert(
                pid.clone(),
                Player {
                    id: pid.clone(),
                    name: format!("Player {}", i + 1),
                    room_id: start_room,
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
            map,
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

pub fn find_room_with_system(state: &GameState, sys: SystemType) -> Option<RoomId> {
    find_room_with_system_in_map(&state.map, sys)
}

pub fn find_room_with_system_in_map(map: &GameMap, sys: SystemType) -> Option<RoomId> {
    for room in map.rooms.values() {
        if room.system == Some(sys) {
            return Some(room.id);
        }
    }
    None
}

pub fn find_empty_rooms(state: &GameState) -> Vec<RoomId> {
    state.map.rooms.values()
        .filter(|r| r.system.is_none())
        .map(|r| r.id)
        .collect()
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
