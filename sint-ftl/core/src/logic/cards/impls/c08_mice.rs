use crate::logic::cards::behavior::CardBehavior;
use crate::types::{GameState, ItemType};

pub struct C08MicePlague;

impl CardBehavior for C08MicePlague {
    fn on_round_end(&self, state: &mut GameState) {
        // Effect: At end of round, lose 2 Peppernuts from Storage (11).
        if let Some(room) = state.map.rooms.get_mut(&11) {
            // Remove up to 2 peppernuts
            let mut removed = 0;
            room.items.retain(|item| {
                if *item == ItemType::Peppernut && removed < 2 {
                    removed += 1;
                    false // Drop
                } else {
                    true // Keep
                }
            });
        }
    }
}
