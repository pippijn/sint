use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, GameState};
use crate::GameError;

pub struct C02StaticNoise;

impl CardBehavior for C02StaticNoise {
    fn validate_action(&self, _state: &GameState, _player_id: &str, action: &Action) -> Result<(), GameError> {
        if let Action::Chat { message } = action {
            // Check for non-emoji characters (simplified: alphabetic)
            if message.chars().any(|c| c.is_alphabetic()) {
                return Err(GameError::Silenced);
            }
        }
        Ok(())
    }
}
