use crate::logic::cards::behavior::CardBehavior;
use crate::types::{GameState, CardType, HazardType, CardId};

pub struct C32WeirdGifts;

impl CardBehavior for C32WeirdGifts {
    fn on_round_end(&self, state: &mut GameState) {
        let mut triggered = false;
        for card in state.active_situations.iter_mut() {
            if card.id == CardId::WeirdGifts {
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
            // 3 Fire in Cargo (4), 1 Fire in Sickbay (10)
            if let Some(room) = state.map.rooms.get_mut(&4) {
                for _ in 0..3 { room.hazards.push(HazardType::Fire); }
            }
            if let Some(room) = state.map.rooms.get_mut(&10) {
                room.hazards.push(HazardType::Fire);
            }
            state.active_situations.retain(|c| c.id != CardId::WeirdGifts);
        }
    }
}
