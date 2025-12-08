use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, GameState};

pub struct C17Listing;

impl CardBehavior for C17Listing {
    fn modify_action_cost(&self, _state: &GameState, _player_id: &str, action: &Action, base_cost: i32) -> i32 {
        // Walking is FREE (0 AP). Actions cost DOUBLE (2 AP).
        match action {
            Action::Move { .. } => 0,
            _ => if base_cost > 0 { base_cost * 2 } else { 0 },
        }
    }
}
