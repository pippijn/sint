use super::ActionHandler;
use crate::logic::cards::get_behavior;
use crate::types::{GameState, HazardType, ItemType, PlayerStatus};
use crate::GameError;

// --- EXTINGUISH ---
pub struct ExtinguishHandler;
impl ActionHandler for ExtinguishHandler {
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

        if !room.hazards.contains(&HazardType::Fire) {
            return Err(GameError::InvalidAction(
                "No fire to extinguish".to_string(),
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

        let p = state
            .players
            .get(player_id)
            .ok_or(GameError::PlayerNotFound)?;
        let has_extinguisher = p.inventory.contains(&ItemType::Extinguisher);
        let room_id = p.room_id;

        if let Some(room) = state.map.rooms.get_mut(&room_id) {
            let limit = if has_extinguisher { 2 } else { 1 };
            let mut removed = 0;
            while removed < limit {
                if let Some(idx) = room.hazards.iter().position(|&h| h == HazardType::Fire) {
                    room.hazards.remove(idx);
                    removed += 1;
                } else {
                    break;
                }
            }
        }
        Ok(())
    }
}

// --- REPAIR ---
pub struct RepairHandler;
impl ActionHandler for RepairHandler {
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

        if !room.hazards.contains(&HazardType::Water) {
            return Err(GameError::InvalidAction("No water to repair".to_string()));
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
            if let Some(idx) = room.hazards.iter().position(|&h| h == HazardType::Water) {
                room.hazards.remove(idx);
            }
        }
        Ok(())
    }
}

// --- REVIVE ---
pub struct ReviveHandler {
    pub target_player: String,
}
impl ActionHandler for ReviveHandler {
    fn validate(&self, state: &GameState, player_id: &str) -> Result<(), GameError> {
        let p = state
            .players
            .get(player_id)
            .ok_or(GameError::PlayerNotFound)?;
        let target = state
            .players
            .get(&self.target_player)
            .ok_or(GameError::PlayerNotFound)?;

        if target.room_id != p.room_id {
            return Err(GameError::InvalidAction(
                "Target not in same room".to_string(),
            ));
        }
        if !target.status.contains(&PlayerStatus::Fainted) {
            return Err(GameError::InvalidAction(
                "Target is not Fainted".to_string(),
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
            target.status.retain(|s| *s != PlayerStatus::Fainted);
            target.hp = 1;
        }
        Ok(())
    }
}

// --- INTERACT ---
pub struct InteractHandler;
impl ActionHandler for InteractHandler {
    fn validate(&self, state: &GameState, player_id: &str) -> Result<(), GameError> {
        let _p = state
            .players
            .get(player_id)
            .ok_or(GameError::PlayerNotFound)?;

        // Check if ANY active situation allows interaction here
        let mut valid = false;
        for card in &state.active_situations {
            if get_behavior(card.id).can_solve(state, player_id) {
                valid = true;
                break;
            }
        }

        if !valid {
            return Err(GameError::InvalidAction(
                "Nothing to Interact with here".to_string(),
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

        let mut solved_idx = None;

        for (i, card) in state.active_situations.iter().enumerate() {
            if get_behavior(card.id).can_solve(state, player_id) {
                solved_idx = Some(i);
                break;
            }
        }

        if let Some(idx) = solved_idx {
            let card_id = state.active_situations[idx].id;

            // Trigger Reward Hook
            get_behavior(card_id).on_solved(state);

            // Pay Cost
            if let Some(sol) = &state.active_situations[idx].solution {
                if let Some(req_item) = &sol.item_cost {
                    if let Some(p) = state.players.get_mut(player_id) {
                        if let Some(pos) = p.inventory.iter().position(|x| x == req_item) {
                            p.inventory.remove(pos);
                        }
                    }
                }
            }
            // Remove Card
            state.active_situations.remove(idx);
        }
        Ok(())
    }
}
