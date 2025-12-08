use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, GameState};
use crate::GameError;

pub struct C43SugarRush;

impl CardBehavior for C43SugarRush {
    fn validate_action(&self, _state: &GameState, _player_id: &str, action: &Action) -> Result<(), GameError> {
        // Negative: Cannons prohibited.
        if let Action::Shoot = action {
            return Err(GameError::InvalidAction("Sugar Rush! Too shaky to shoot.".to_string()));
        }
        Ok(())
    }

    fn modify_action_cost(&self, _state: &GameState, _player_id: &str, action: &Action, base_cost: i32) -> i32 {
        // Positive: Move 1 room extra free.
        // Simplified: Move is free.
        if let Action::Move { .. } = action {
            0
        } else {
            base_cost
        }
    }
}
