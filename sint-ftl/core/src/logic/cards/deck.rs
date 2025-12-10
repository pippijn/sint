use super::get_behavior;
use crate::types::*;
use rand::{rngs::StdRng, seq::SliceRandom};

pub fn initialize_deck(rng: &mut StdRng) -> Vec<Card> {
    let mut deck = crate::logic::cards::registry::get_all_cards();
    deck.shuffle(rng);
    deck
}

pub fn draw_card(state: &mut GameState) {
    if let Some(card) = state.deck.pop() {
        state.latest_event = Some(card.clone());

        match card.card_type {
            CardType::Flash => {
                get_behavior(card.id).on_activate(state);
                state.discard.push(card);
            }
            CardType::Situation | CardType::Timebomb { .. } => {
                state.active_situations.push(card.clone());
                get_behavior(card.id).on_activate(state);
            }
        }
    }
}
