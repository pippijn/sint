use crate::logic::cards::behavior::CardBehavior;
use crate::types::{GameState, CardType, CardId};

pub struct C37Recipe;

impl CardBehavior for C37Recipe {
    fn on_round_end(&self, state: &mut GameState) {
        // Timebomb.
        // If solved (Interact at Bow), reward is given.
        // If time runs out, nothing happens (Recipe lost).
        for card in state.active_situations.iter_mut() {
            if card.id == CardId::Recipe {
                if let CardType::Timebomb { rounds_left } = &mut card.card_type {
                    if *rounds_left > 0 {
                        *rounds_left -= 1;
                    }
                }
            }
        }
        // Cleanup if empty?
        state.active_situations.retain(|c| {
            if c.id == CardId::Recipe {
                if let CardType::Timebomb { rounds_left } = c.card_type {
                    rounds_left > 0
                } else { true }
            } else { true }
        });
    }
}
