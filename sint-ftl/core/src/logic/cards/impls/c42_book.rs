use crate::logic::cards::behavior::CardBehavior;
use crate::types::{GameState, CardType, CardId};

pub struct C42TheBook;

impl CardBehavior for C42TheBook {
    fn on_round_end(&self, state: &mut GameState) {
        // "Reward" triggered by solving? 
        // Solution logic is "Interact". We don't track if it was solved here.
        // But if the card is REMOVED by solution, the effect triggers?
        // Card text: "Mission: Get book... REWARD: Enemy skips NEXT ATTACK."
        // Solution logic in `resolution.rs` removes the card.
        // We need to hook into removal?
        // Or `on_round_end` just ticks timebomb.
        // If solved, `resolution.rs` removes it. 
        // We need a way to apply reward upon resolution.
        // But `CardBehavior` doesn't have `on_solved`.
        
        // Timebomb tick
        for card in state.active_situations.iter_mut() {
            if card.id == CardId::TheBook {
                if let CardType::Timebomb { rounds_left } = &mut card.card_type {
                    if *rounds_left > 0 {
                        *rounds_left -= 1;
                    }
                }
            }
        }
        state.active_situations.retain(|c| {
            if c.id == CardId::TheBook {
                if let CardType::Timebomb { rounds_left } = c.card_type {
                    rounds_left > 0
                } else { true }
            } else { true }
        });
    }
}
