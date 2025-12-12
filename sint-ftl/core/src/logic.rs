pub mod actions;
pub mod cards;
pub mod handlers;
pub mod map_gen;
pub mod pathfinding;
pub mod resolution;

pub use actions::apply_action;

use crate::types::*;
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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

pub struct GameLogic;

impl GameLogic {
    pub fn new_game(player_ids: Vec<String>, seed: u64) -> GameState {
        Self::new_game_with_layout(player_ids, seed, MapLayout::Star)
    }

    pub fn new_game_with_layout(
        player_ids: Vec<String>,
        seed: u64,
        layout: MapLayout,
    ) -> GameState {
        let map = map_gen::generate_map(layout);

        // Determine Start Room (Dormitory)
        let start_room = find_room_with_system_in_map(&map, SystemType::Dormitory).unwrap_or(0);

        let mut players = BTreeMap::new();
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
            hull_integrity: MAX_HULL,
            boss_level: 0,
            layout,
            map,
            players,
            enemy: get_boss(0),
            chat_log: vec![],
            shields_active: false,
            evasion_active: false,
            is_resting: false,
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
    state
        .map
        .rooms
        .values()
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
            name: (*name).to_owned(),
            hp: *hp,
            max_hp: *hp,
            state: EnemyState::Active,
            next_attack: None,
        }
    } else {
        Enemy {
            name: "Unknown Threat".to_owned(),
            hp: 50,
            max_hp: 50,
            state: EnemyState::Active,
            next_attack: None,
        }
    }
}
