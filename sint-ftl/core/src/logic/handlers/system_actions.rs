use super::ActionHandler;
use crate::GameError;
use crate::types::{ChatMessage, EnemyState, GameState, ItemType, PlayerStatus, SystemType};
use log::info;
use rand::{Rng, SeedableRng, rngs::StdRng};

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
            let correct_id =
                crate::logic::find_room_with_system_in_map(&state.map, SystemType::Kitchen)
                    .map(|id| id.to_string())
                    .unwrap_or("?".to_string());
            return Err(GameError::InvalidAction(format!(
                "Bake requires Kitchen (Room {}), but you are in {} ({})",
                correct_id, room.name, room.id
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
        let p = state
            .players
            .get(player_id)
            .ok_or(GameError::PlayerNotFound)?;
        let room_id = p.room_id;

        if let Some(room) = state.map.rooms.get_mut(&room_id) {
            room.items
                .extend(std::iter::repeat_n(ItemType::Peppernut, 3));
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
            let correct_id =
                crate::logic::find_room_with_system_in_map(&state.map, SystemType::Cannons)
                    .map(|id| id.to_string())
                    .unwrap_or("?".to_string());
            return Err(GameError::InvalidAction(format!(
                "Shoot requires Cannons (Room {}), but you are in {} ({})",
                correct_id, room.name, room.id
            )));
        }
        if !room.hazards.is_empty() {
            return Err(GameError::RoomBlocked);
        }
        if !p.inventory.contains(&ItemType::Peppernut) {
            return Err(GameError::InvalidAction(
                "No ammo (Peppernut) to shoot".to_owned(),
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
        let p = state
            .players
            .get_mut(player_id)
            .ok_or(GameError::PlayerNotFound)?;
        if let Some(idx) = p.inventory.iter().position(|i| *i == ItemType::Peppernut) {
            p.inventory.remove(idx);
        }

        // Calculate Hit
        let hit = if simulation {
            false // In simulation, we don't apply damage to avoid side effects in ghost-view
        } else {
            let mut rng = StdRng::seed_from_u64(state.rng_seed);
            let roll: u32 = rng.random_range(1..=6);
            state.rng_seed = rng.random();

            use crate::logic::cards::get_behavior;

            let mut threshold = 3;
            for card in &state.active_situations {
                let t = get_behavior(card.id).get_hit_threshold(state);
                if t > threshold {
                    threshold = t;
                }
            }
            if roll < threshold {
                state.chat_log.push(ChatMessage {
                    sender: "SYSTEM".to_owned(),
                    text: format!("{} missed the shot! (Rolled {})", player_id, roll),
                    timestamp: 0,
                });
            }
            roll >= threshold
        };

        if hit {
            state.enemy.hp -= 1;

            if state.enemy.hp <= 0 {
                if state.boss_level >= crate::logic::MAX_BOSS_LEVEL - 1 {
                    state.phase = crate::types::GamePhase::Victory;
                    state.chat_log.push(ChatMessage {
                        sender: "SYSTEM".to_owned(),
                        text: "VICTORY! All bosses defeated!".to_owned(),
                        timestamp: 0,
                    });
                } else {
                    state.enemy.state = EnemyState::Defeated;
                    state.chat_log.push(ChatMessage {
                        sender: "SYSTEM".to_owned(),
                        text: format!("{} Defeated! Taking a breather...", state.enemy.name),
                        timestamp: 0,
                    });
                }
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

        if room.system != Some(SystemType::Bridge) {
            let correct_id =
                crate::logic::find_room_with_system_in_map(&state.map, SystemType::Bridge)
                    .map(|id| id.to_string())
                    .unwrap_or("?".to_string());
            return Err(GameError::InvalidAction(format!(
                "Raise Shields requires Bridge (Room {}), but you are in {} ({})",
                correct_id, room.name, room.id
            )));
        }
        if !room.hazards.is_empty() {
            return Err(GameError::RoomBlocked);
        }
        if state.shields_active {
            return Err(GameError::InvalidAction(
                "Shields are already active".to_owned(),
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

        if room.system != Some(SystemType::Engine) {
            let correct_id =
                crate::logic::find_room_with_system_in_map(&state.map, SystemType::Engine)
                    .map(|id| id.to_string())
                    .unwrap_or("?".to_string());
            return Err(GameError::InvalidAction(format!(
                "Evasive Maneuvers requires Engine (Room {}), but you are in {} ({})",
                correct_id, room.name, room.id
            )));
        }
        if !room.hazards.is_empty() {
            return Err(GameError::RoomBlocked);
        }
        if state.evasion_active {
            return Err(GameError::InvalidAction(
                "Evasive maneuvers are already active".to_owned(),
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
            let correct_id =
                crate::logic::find_room_with_system_in_map(&state.map, SystemType::Bow)
                    .map(|id| id.to_string())
                    .unwrap_or("?".to_string());
            return Err(GameError::InvalidAction(format!(
                "Lookout requires The Bow (Room {}), but you are in {} ({})",
                correct_id, room.name, room.id
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

        let card_id = state.deck.last();
        let msg = if let Some(c_id) = card_id {
            let c = crate::logic::cards::registry::get_behavior(*c_id).get_struct();
            format!(
                "LOOKOUT REPORT: The next event is '{}' ({})",
                c.title, c.description
            )
        } else {
            "LOOKOUT REPORT: The horizon is clear (Deck Empty).".to_owned()
        };

        state.chat_log.push(ChatMessage {
            sender: "SYSTEM".to_owned(),
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
            let correct_id =
                crate::logic::find_room_with_system_in_map(&state.map, SystemType::Sickbay)
                    .map(|id| id.to_string())
                    .unwrap_or("?".to_string());
            return Err(GameError::InvalidAction(format!(
                "First Aid requires Sickbay (Room {}), but you are in {} ({})",
                correct_id, room.name, room.id
            )));
        }
        if !room.hazards.is_empty() {
            return Err(GameError::RoomBlocked);
        }

        let target = state
            .players
            .get(&self.target_player)
            .ok_or(GameError::PlayerNotFound)?;

        if target.status.contains(&PlayerStatus::Fainted) {
            return Err(GameError::InvalidAction(
                "Cannot use First Aid on a Fainted player. Use Revive instead.".to_owned(),
            ));
        }

        let is_self = self.target_player == player_id;
        let is_adjacent = room.neighbors.contains(&target.room_id);
        let is_here = target.room_id == p.room_id;

        if !is_self && !is_adjacent && !is_here {
            return Err(GameError::InvalidAction(
                "Target for First Aid must be self or in adjacent/same room".to_owned(),
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
        if let Some(target) = state.players.get_mut(&self.target_player)
            && target.hp < 3
        {
            target.hp += 1;
        }
        Ok(())
    }
}
