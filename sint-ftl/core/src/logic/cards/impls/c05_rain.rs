use crate::logic::cards::behavior::CardBehavior;
use crate::types::{GameState, ItemType};

pub struct C05PeppernutRain;

impl CardBehavior for C05PeppernutRain {
    fn on_activate(&self, state: &mut GameState) {
        // Effect: +2 Peppernuts dropped in every occupied room.
        let occupied_rooms: Vec<u32> = state
            .players
            .values()
            .map(|p| p.room_id)
            .collect::<std::collections::HashSet<_>>() // Dedup
            .into_iter()
            .collect();

        for rid in occupied_rooms {
            if let Some(room) = state.map.rooms.get_mut(&rid) {
                room.items.push(ItemType::Peppernut);
                room.items.push(ItemType::Peppernut);
            }
        }
    }
}
