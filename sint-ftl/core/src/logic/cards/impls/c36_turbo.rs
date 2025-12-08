use crate::logic::cards::behavior::CardBehavior;
use crate::types::{GameState, CardType, HazardType, CardId};

pub struct C36TurboMode;

impl CardBehavior for C36TurboMode {
    fn on_round_end(&self, state: &mut GameState) {
        let mut triggered = false;
        for card in state.active_situations.iter_mut() {
            if card.id == CardId::TurboMode {
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
            // Explosion
            if let Some(room) = state.map.rooms.get_mut(&5) {
                room.hazards.push(HazardType::Fire);
                room.hazards.push(HazardType::Fire);
            }
            if let Some(room) = state.map.rooms.get_mut(&7) {
                room.hazards.push(HazardType::Fire);
            }
            state.active_situations.retain(|c| c.id != CardId::TurboMode);
        }
    }
    
    // Advantage: 3 AP. Hook into `reset_ap` logic?
    // We don't have a hook for AP reset. We set it to 2 in `advance_phase`.
    // We should add a `modify_max_ap` hook or similar.
}
