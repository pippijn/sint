use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, GameState};
use crate::GameError;

pub struct C13AnchorStuck;

impl CardBehavior for C13AnchorStuck {
    fn validate_action(&self, _state: &GameState, _player_id: &str, action: &Action) -> Result<(), GameError> {
        if let Action::EvasiveManeuvers = action {
            return Err(GameError::InvalidAction(
                "Anchor Stuck! Cannot use Evasive Maneuvers.".to_string(),
            ));
        }
        Ok(())
    }
}
