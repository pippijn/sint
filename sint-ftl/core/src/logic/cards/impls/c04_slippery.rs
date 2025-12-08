use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, GameState};

pub struct C04SlipperyDeck;

impl CardBehavior for C04SlipperyDeck {
    fn modify_action_cost(&self, _state: &GameState, _player_id: &str, action: &Action, base_cost: i32) -> i32 {
        match action {
            Action::Move { .. } => 0, // Moves are free
            _ => if base_cost > 0 { base_cost + 1 } else { 0 }, // Actions cost +1 (unless free)
        }
    }
}
