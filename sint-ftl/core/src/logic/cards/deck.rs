use super::get_behavior;
use crate::types::*;
use rand::{rngs::StdRng, seq::SliceRandom};

pub fn initialize_deck(rng: &mut StdRng) -> Vec<CardId> {
    let mut deck = crate::logic::cards::registry::get_all_ids();
    deck.shuffle(rng);
    deck
}

pub fn draw_card(state: &mut GameState) {
    if let Some(card_id) = state.deck.pop() {
        let card = get_behavior(card_id).get_struct();
        state.latest_event = Some(card.clone());

        match card.card_type {
            CardType::Flash => {
                get_behavior(card_id).on_activate(state);
                state.discard.push(card_id);
            }
            CardType::Situation | CardType::Timebomb { .. } => {
                state.active_situations.push(card.clone());
                get_behavior(card_id).on_activate(state);
            }
        }
    }
}
