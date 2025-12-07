use crate::types::*;
use rand::{Rng, SeedableRng};
use rand::seq::SliceRandom;
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

        // Initialize RNG for shuffling
        let mut rng = StdRng::seed_from_u64(seed);
        let next_seed = rng.gen();

        let mut state = GameState {
            sequence_id: 0,
            rng_seed: next_seed,
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
            latest_event: None,
            deck: Self::initialize_deck(&mut rng),
            discard: vec![],
        };
        
        // Draw initial card
        Self::draw_card(&mut state);
        state
    }

    fn initialize_deck(rng: &mut StdRng) -> Vec<Card> {
        let mut deck = vec![
            Card {
                id: "C01".to_string(),
                title: "Afternoon Nap".to_string(),
                description: "The Reader falls asleep. Cannot spend AP.".to_string(),
                card_type: CardType::Situation,
                options: vec![],
                solution: Some(CardSolution {
                    room_id: None, 
                    ap_cost: 1, 
                    item_cost: None,
                    required_players: 1
                })
            },
            Card {
                id: "C05".to_string(),
                title: "Peppernut Rain".to_string(),
                description: "+2 Peppernuts for everyone.".to_string(),
                card_type: CardType::Flash,
                options: vec![],
                solution: None
            }
        ];
        
        deck.shuffle(rng);
        deck
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
                
                // Check phase advance if ALL players passed?
                // For now, only VoteReady does it.
            },
            
            Action::VoteReady { ready } => {
                player.is_ready = ready;
                
                // Check Consensus
                let all_ready = state.players.values().all(|p| p.is_ready);
                if all_ready {
                    state = Self::advance_phase(state)?;
                }
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

    fn advance_phase(mut state: GameState) -> Result<GameState, GameError> {
        match state.phase {
            GamePhase::MorningReport => {
                state.phase = GamePhase::EnemyTelegraph;
                
                // Archive the event
                state.latest_event = None;
                
                // Generate telegraph
                // For now, simple logic
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
                state.phase = GamePhase::EnemyAction;
                // Run Logic
                Self::resolve_enemy_attack(&mut state);
                Self::resolve_hazards(&mut state);
                
                for p in state.players.values_mut() { p.is_ready = false; }
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
                
                Self::draw_card(&mut state);
                for p in state.players.values_mut() { p.is_ready = false; }
            },
            _ => {}
        }
        Ok(state)
    }

    fn resolve_enemy_attack(state: &mut GameState) {
        if let Some(attack) = &state.enemy.next_attack {
            // Check Evasion (TODO: Add Evasion Flag to State)
            // Check Shields (TODO: Add Shield Flag to State)
            
            // Hit!
            if let Some(room) = state.map.rooms.get_mut(&attack.target_room) {
                 match attack.effect {
                     AttackEffect::Fireball => {
                         room.hazards.push(HazardType::Fire);
                         state.hull_integrity -= 1; // Direct hit damage?
                     },
                     AttackEffect::Leak => {
                         room.hazards.push(HazardType::Water);
                     },
                     _ => {}
                 }
            }
        }
        state.enemy.next_attack = None;
    }

    fn resolve_hazards(state: &mut GameState) {
        let mut fire_spreads = vec![];
        let mut rng = StdRng::seed_from_u64(state.rng_seed);
        
        // 1. Damage Players & Hull
        for room in state.map.rooms.values() {
            let has_fire = room.hazards.contains(&HazardType::Fire);
            
            if has_fire {
                state.hull_integrity -= 1;
                
                // Spread Chance? (Simple: if > 1 fire, spread to neighbors)
                if room.hazards.iter().filter(|&h| *h == HazardType::Fire).count() >= 2 {
                    for &neighbor in &room.neighbors {
                        if rng.gen_bool(0.5) { // 50% chance to spread
                            fire_spreads.push(neighbor);
                        }
                    }
                }
            }
        }
        
        // Apply Player Damage separately to avoid borrow issues
        for p in state.players.values_mut() {
            if let Some(room) = state.map.rooms.get(&p.room_id) {
                if room.hazards.contains(&HazardType::Fire) {
                    p.hp -= 1;
                    if p.hp <= 0 {
                        p.status.push(PlayerStatus::Fainted);
                    }
                }
            }
        }

        // 2. Apply Spreads
        for room_id in fire_spreads {
            if let Some(room) = state.map.rooms.get_mut(&room_id) {
                if !room.hazards.contains(&HazardType::Fire) {
                    room.hazards.push(HazardType::Fire);
                }
            }
        }
        
        state.rng_seed = rng.gen();
    }

    fn draw_card(state: &mut GameState) {
        // Simple draw logic
        if let Some(card) = state.deck.pop() {
            state.latest_event = Some(card.clone());

            match card.card_type {
                 CardType::Flash => {
                     // Apply flash effect (placeholder)
                     // e.g. "Rain" -> Add items
                     state.discard.push(card);
                 },
                 CardType::Situation | CardType::Timebomb { .. } => {
                     state.active_situations.push(card);
                 }
            }
        } else {
            // Reshuffle discard?
        }
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
