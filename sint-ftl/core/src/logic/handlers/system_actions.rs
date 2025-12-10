use super::ActionHandler;
use crate::types::{ChatMessage, GameState, ItemType, SystemType};
use crate::GameError;
use log::info;
use rand::{rngs::StdRng, Rng, SeedableRng};

// --- BAKE ---
pub struct BakeHandler;
impl ActionHandler for BakeHandler {
    fn validate(&self, state: &GameState, player_id: &str) -> Result<(), GameError> {
        let p = state
            .players
            .get(player_id)
            .ok_or(GameError::PlayerNotFound)?;
        let room = state
            .map
            .rooms
            .get(&p.room_id)
            .ok_or(GameError::RoomNotFound)?;

        if room.system != Some(SystemType::Kitchen) {
            return Err(GameError::InvalidAction(format!(
                "Bake requires Kitchen (6), but you are in {} ({})",
                room.name, room.id
            )));
        }
        if !room.hazards.is_empty() {
            return Err(GameError::RoomBlocked);
        }
        Ok(())
    }

    fn execute(
        &self,
        state: &mut GameState,
        player_id: &str,
        _simulation: bool,
    ) -> Result<(), GameError> {
        self.validate(state, player_id)?;
        let p = state.players.get(player_id).unwrap();
        let room_id = p.room_id;

        if let Some(room) = state.map.rooms.get_mut(&room_id) {
            room.items.push(ItemType::Peppernut);
            room.items.push(ItemType::Peppernut);
            room.items.push(ItemType::Peppernut);
        }
        Ok(())
    }
}

// --- SHOOT ---
pub struct ShootHandler;
impl ActionHandler for ShootHandler {
    fn validate(&self, state: &GameState, player_id: &str) -> Result<(), GameError> {
        let p = state
            .players
            .get(player_id)
            .ok_or(GameError::PlayerNotFound)?;
        let room = state
            .map
            .rooms
            .get(&p.room_id)
            .ok_or(GameError::RoomNotFound)?;

        if room.system != Some(SystemType::Cannons) {
            return Err(GameError::InvalidAction(format!(
                "Shoot requires Cannons (8), but you are in {} ({})",
                room.name, room.id
            )));
        }
        if !room.hazards.is_empty() {
            return Err(GameError::RoomBlocked);
        }
        if !p.inventory.contains(&ItemType::Peppernut) {
            return Err(GameError::InvalidAction(
                "No ammo (Peppernut) to shoot".to_string(),
            ));
        }
        Ok(())
    }

    fn execute(
        &self,
        state: &mut GameState,
        player_id: &str,
        simulation: bool,
    ) -> Result<(), GameError> {
        self.validate(state, player_id)?;

        // Consume Ammo
        let p = state.players.get_mut(player_id).unwrap();
        if let Some(idx) = p.inventory.iter().position(|i| *i == ItemType::Peppernut) {
            p.inventory.remove(idx);
        }

        if !simulation {
            // Calculate Hit
            let mut rng = StdRng::seed_from_u64(state.rng_seed);
            let roll: u32 = rng.gen_range(1..=6);
            state.rng_seed = rng.gen();

            use crate::logic::cards::get_behavior;

            let mut threshold = 3;
            for card in &state.active_situations {
                let t = get_behavior(card.id).get_hit_threshold(state);
                if t > threshold {
                    threshold = t;
                }
            }

            if roll >= threshold {
                state.enemy.hp -= 1;

                if state.enemy.hp <= 0 {
                    state.boss_level += 1;
                    if state.boss_level >= crate::logic::MAX_BOSS_LEVEL {
                        state.phase = crate::types::GamePhase::Victory;
                        state.chat_log.push(ChatMessage {
                            sender: "SYSTEM".to_string(),
                            text: "VICTORY! All bosses defeated!".to_string(),
                            timestamp: 0,
                        });
                    } else {
                        state.enemy = crate::logic::get_boss(state.boss_level);
                        state.chat_log.push(ChatMessage {
                            sender: "SYSTEM".to_string(),
                            text: format!("Enemy Defeated! approaching: {}", state.enemy.name),
                            timestamp: 0,
                        });
                    }
                }
            } else {
                // Miss
                state.chat_log.push(ChatMessage {
                    sender: "SYSTEM".to_string(),
                    text: format!("{} missed the shot! (Rolled {})", player_id, roll),
                    timestamp: 0,
                });
            }
        }

        Ok(())
    }
}

// --- RAISE SHIELDS ---
pub struct RaiseShieldsHandler;
impl ActionHandler for RaiseShieldsHandler {
    fn validate(&self, state: &GameState, player_id: &str) -> Result<(), GameError> {
        let p = state
            .players
            .get(player_id)
            .ok_or(GameError::PlayerNotFound)?;
        let room = state
            .map
            .rooms
            .get(&p.room_id)
            .ok_or(GameError::RoomNotFound)?;

        if room.system != Some(SystemType::Engine) {
            return Err(GameError::InvalidAction(format!(
                "Raise Shields requires Engine (5), but you are in {} ({})",
                room.name, room.id
            )));
        }
        if !room.hazards.is_empty() {
            return Err(GameError::RoomBlocked);
        }
        Ok(())
    }

    fn execute(
        &self,
        state: &mut GameState,
        player_id: &str,
        _simulation: bool,
    ) -> Result<(), GameError> {
        self.validate(state, player_id)?;
        state.shields_active = true;
        info!("{} raised shields!", player_id);
        Ok(())
    }

    fn base_cost(&self) -> i32 {
        2
    }
}

// --- EVASIVE MANEUVERS ---
pub struct EvasiveManeuversHandler;
impl ActionHandler for EvasiveManeuversHandler {
    fn validate(&self, state: &GameState, player_id: &str) -> Result<(), GameError> {
        let p = state
            .players
            .get(player_id)
            .ok_or(GameError::PlayerNotFound)?;
        let room = state
            .map
            .rooms
            .get(&p.room_id)
            .ok_or(GameError::RoomNotFound)?;

        if room.system != Some(SystemType::Bridge) {
            return Err(GameError::InvalidAction(format!(
                "Evasive Maneuvers requires Bridge (9), but you are in {} ({})",
                room.name, room.id
            )));
        }
        if !room.hazards.is_empty() {
            return Err(GameError::RoomBlocked);
        }
        Ok(())
    }

    fn execute(
        &self,
        state: &mut GameState,
        player_id: &str,
        _simulation: bool,
    ) -> Result<(), GameError> {
        self.validate(state, player_id)?;
        state.evasion_active = true;
        info!("{} engaged evasive maneuvers!", player_id);
        Ok(())
    }

    fn base_cost(&self) -> i32 {
        2
    }
}

// --- LOOKOUT ---
pub struct LookoutHandler;
impl ActionHandler for LookoutHandler {
    fn validate(&self, state: &GameState, player_id: &str) -> Result<(), GameError> {
        let p = state
            .players
            .get(player_id)
            .ok_or(GameError::PlayerNotFound)?;
        let room = state
            .map
            .rooms
            .get(&p.room_id)
            .ok_or(GameError::RoomNotFound)?;

        if room.system != Some(SystemType::Bow) {
            return Err(GameError::InvalidAction(format!(
                "Lookout requires The Bow (2), but you are in {} ({})",
                room.name, room.id
            )));
        }
        if !room.hazards.is_empty() {
            return Err(GameError::RoomBlocked);
        }
        Ok(())
    }

    fn execute(
        &self,
        state: &mut GameState,
        player_id: &str,
        _simulation: bool,
    ) -> Result<(), GameError> {
        self.validate(state, player_id)?;

        let card = state.deck.last();
        let msg = if let Some(c) = card {
            format!(
                "LOOKOUT REPORT: The next event is '{}' ({})",
                c.title, c.description
            )
        } else {
            "LOOKOUT REPORT: The horizon is clear (Deck Empty).".to_string()
        };

        state.chat_log.push(ChatMessage {
            sender: "SYSTEM".to_string(),
            text: msg,
            timestamp: 0,
        });
        Ok(())
    }
}

// --- FIRST AID ---
pub struct FirstAidHandler {
    pub target_player: String,
}
impl ActionHandler for FirstAidHandler {
    fn validate(&self, state: &GameState, player_id: &str) -> Result<(), GameError> {
        let p = state
            .players
            .get(player_id)
            .ok_or(GameError::PlayerNotFound)?;
        let room = state
            .map
            .rooms
            .get(&p.room_id)
            .ok_or(GameError::RoomNotFound)?;

        if room.system != Some(SystemType::Sickbay) {
            return Err(GameError::InvalidAction(format!(
                "First Aid requires Sickbay (10), but you are in {} ({})",
                room.name, room.id
            )));
        }
        if !room.hazards.is_empty() {
            return Err(GameError::RoomBlocked);
        }

        let target = state
            .players
            .get(&self.target_player)
            .ok_or(GameError::PlayerNotFound)?;
        let is_self = self.target_player == player_id;
        let is_adjacent = room.neighbors.contains(&target.room_id);
        let is_here = target.room_id == p.room_id;

        if !is_self && !is_adjacent && !is_here {
            return Err(GameError::InvalidAction(
                "Target for First Aid must be self or in adjacent/same room".to_string(),
            ));
        }
        Ok(())
    }

    fn execute(
        &self,
        state: &mut GameState,
        player_id: &str,
        _simulation: bool,
    ) -> Result<(), GameError> {
        self.validate(state, player_id)?;
        if let Some(target) = state.players.get_mut(&self.target_player) {
            if target.hp < 3 {
                target.hp += 1;
            }
        }
        Ok(())
    }
}
