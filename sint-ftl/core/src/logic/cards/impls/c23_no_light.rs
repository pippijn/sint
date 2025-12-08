use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, GameState};
use crate::GameError;

pub struct C23NoLight;

impl CardBehavior for C23NoLight {
    fn validate_action(&self, _state: &GameState, _player_id: &str, action: &Action) -> Result<(), GameError> {
        if let Action::Shoot = action {
            return Err(GameError::InvalidAction(
                "No Light! Cannons can't aim.".to_string(),
            ));
        }
        Ok(())
    }
}
