use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, GameState};
use crate::GameError;

pub struct C18CloggedPipe;

impl CardBehavior for C18CloggedPipe {
    fn validate_action(&self, _state: &GameState, _player_id: &str, action: &Action) -> Result<(), GameError> {
        if let Action::Bake = action {
            return Err(GameError::InvalidAction(
                "Clogged Pipe! Cannot Bake.".to_string(),
            ));
        }
        Ok(())
    }
}
