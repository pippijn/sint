use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, GameState};
use crate::GameError;

pub struct C31Blockade;

impl CardBehavior for C31Blockade {
    fn validate_action(&self, state: &GameState, player_id: &str, action: &Action) -> Result<(), GameError> {
        // Door to Cannons (8) is closed.
        // No one can enter or exit.
        if let Action::Move { to_room } = action {
            if *to_room == 8 {
                return Err(GameError::InvalidAction("Blockade! Cannot enter Room 8.".to_string()));
            }
            if let Some(p) = state.players.get(player_id) {
                if p.room_id == 8 {
                    return Err(GameError::InvalidAction("Blockade! Cannot exit Room 8.".to_string()));
                }
            }
        }
        Ok(())
    }
}
