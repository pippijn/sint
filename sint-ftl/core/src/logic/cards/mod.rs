pub mod behavior;
pub mod deck;
pub mod impls;
pub mod registry;

pub use behavior::CardBehavior;
pub use deck::{draw_card, initialize_deck};
pub use registry::get_behavior;

use crate::types::{CardSentiment, GameState};

pub fn find_solvable_card(state: &GameState, player_id: &str) -> Option<usize> {
    let mut solved_idx = None;

    // Prioritize Negative situations (threats)
    for (i, card) in state.active_situations.iter().enumerate() {
        if get_behavior(card.id).can_solve(state, player_id)
            && get_behavior(card.id).get_sentiment() == CardSentiment::Negative
        {
            solved_idx = Some(i);
            break;
        }
    }

    // Fallback to any solvable situation
    if solved_idx.is_none() {
        for (i, card) in state.active_situations.iter().enumerate() {
            if get_behavior(card.id).can_solve(state, player_id) {
                solved_idx = Some(i);
                break;
            }
        }
    }

    solved_idx
}
