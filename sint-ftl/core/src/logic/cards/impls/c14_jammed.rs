use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, GameState};
use crate::GameError;

pub struct C14JammedCannon;

impl CardBehavior for C14JammedCannon {
    fn validate_action(&self, _state: &GameState, _player_id: &str, action: &Action) -> Result<(), GameError> {
        if let Action::Shoot = action {
            return Err(GameError::InvalidAction(
                "Cannon Jammed! Cannot Shoot.".to_string(),
            ));
        }
        Ok(())
    }
}
