use crate::logic::cards::behavior::CardBehavior;
use crate::types::{GameState, CardType, ItemType, CardId};

pub struct C35Stowaway;

impl CardBehavior for C35Stowaway {
    fn on_round_end(&self, state: &mut GameState) {
        let mut triggered = false;
        for card in state.active_situations.iter_mut() {
            if card.id == CardId::Stowaway {
                if let CardType::Timebomb { rounds_left } = &mut card.card_type {
                    if *rounds_left > 0 {
                        *rounds_left -= 1;
                        if *rounds_left == 0 {
                            triggered = true;
                        }
                    }
                }
            }
        }
        
        if triggered {
            // Theft: Lose all peppernuts
            for p in state.players.values_mut() {
                p.inventory.retain(|i| *i != ItemType::Peppernut);
            }
            state.active_situations.retain(|c| c.id != CardId::Stowaway);
        }
    }
}
