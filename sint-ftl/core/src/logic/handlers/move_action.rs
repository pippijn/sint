use super::ActionHandler;
use crate::GameError;
use crate::types::GameState;

pub struct MoveHandler {
    pub to_room: u32,
}

impl ActionHandler for MoveHandler {
    fn validate(&self, state: &GameState, player_id: &str) -> Result<(), GameError> {
        let p = state
            .players
            .get(player_id)
            .ok_or(GameError::PlayerNotFound)?;
        let current_room_id = p.room_id;
        let room = state
            .map
            .rooms
            .get(&current_room_id)
            .ok_or(GameError::RoomNotFound)?;

        if !room.neighbors.contains(&self.to_room) {
            return Err(GameError::InvalidMove);
        }
        Ok(())
    }

    fn execute(
        &self,
        state: &mut GameState,
        player_id: &str,
        _simulation: bool,
    ) -> Result<(), GameError> {
        // Validation (can be skipped if performance is critical and validation guaranteed upstream,
        // but safe to keep for robust core)
        self.validate(state, player_id)?;

        if let Some(p) = state.players.get_mut(player_id) {
            p.room_id = self.to_room;
        }
        Ok(())
    }
}
