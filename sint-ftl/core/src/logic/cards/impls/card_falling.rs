use crate::logic::cards::behavior::CardBehavior;
use crate::types::{GameState, HazardType, ItemType};

pub struct FallingGiftCard;

use crate::types::{Card, CardId, CardType};

impl CardBehavior for FallingGiftCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::FallingGift,
            title: "Falling Gift".to_string(),
            description: "Leak in Cargo (4). +2 Peppernuts in Cargo.".to_string(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        // Effect: Leak in Cargo (4). 1 Water.
        if let Some(room) = state.map.rooms.get_mut(&4) {
            room.hazards.push(HazardType::Water);
        }

        // Bonus: 2 Peppernuts in Room 4.
        if let Some(room) = state.map.rooms.get_mut(&4) {
            room.items.push(ItemType::Peppernut);
            room.items.push(ItemType::Peppernut);
        }
    }
}
