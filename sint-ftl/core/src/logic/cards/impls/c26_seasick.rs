use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, GameState};
use crate::GameError;

pub struct C26Seasick;

impl CardBehavior for C26Seasick {
    fn validate_action(&self, _state: &GameState, _player_id: &str, _action: &Action) -> Result<(), GameError> {
        // Effect: You may EITHER Walk OR do Actions (not both).
        // Check if player has already spent AP on a conflicting type?
        // Hard to track "previous action type" without history.
        // Heuristic: If AP < Max, check what we did?
        // We'll skip strict validation in Planning for now, as it requires tracking intent across the batch.
        Ok(())
    }
}
