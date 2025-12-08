use crate::logic::cards::behavior::CardBehavior;
use crate::types::{GameState, HazardType};

pub struct LeakCard;

use crate::types::{Card, CardId, CardType};

impl CardBehavior for LeakCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Leak,
            title: "Leak!".to_string(),
            description: "Spawn 1 Water in the Cargo Room (4).".to_string(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        // Effect: Spawn 1 Water in the Cargo Room (4).
        if let Some(room) = state.map.rooms.get_mut(&4) {
            room.hazards.push(HazardType::Water);
        }
    }
}
