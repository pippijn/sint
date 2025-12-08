use crate::logic::cards::behavior::CardBehavior;
use crate::types::{GameState, ItemType};

pub struct PeppernutRainCard;

use crate::types::{Card, CardId, CardType};

impl CardBehavior for PeppernutRainCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::PeppernutRain,
            title: "Peppernut Rain".to_string(),
            description: "+2 Peppernuts dropped in every occupied room.".to_string(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        }
    }

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
