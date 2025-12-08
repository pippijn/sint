use crate::logic::cards::behavior::CardBehavior;
use crate::types::{GameState, CardType, CardId};

pub struct C38GoldenNut;

impl CardBehavior for C38GoldenNut {
    fn on_round_end(&self, state: &mut GameState) {
        for card in state.active_situations.iter_mut() {
            if card.id == CardId::GoldenNut {
                if let CardType::Timebomb { rounds_left } = &mut card.card_type {
                    if *rounds_left > 0 {
                        *rounds_left -= 1;
                    }
                }
            }
        }
        state.active_situations.retain(|c| {
            if c.id == CardId::GoldenNut {
                if let CardType::Timebomb { rounds_left } = c.card_type {
                    rounds_left > 0
                } else { true }
            } else { true }
        });
    }
}
