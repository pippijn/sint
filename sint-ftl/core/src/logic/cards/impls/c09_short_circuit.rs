use crate::logic::cards::behavior::CardBehavior;
use crate::types::{GameState, HazardType};

pub struct C09ShortCircuit;

use crate::types::{Card, CardId, CardType};

impl CardBehavior for C09ShortCircuit {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::ShortCircuit,
            title: "Short Circuit".to_string(),
            description: "Spawn 1 Fire in the Engine Room (5).".to_string(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        // Effect: Spawn 1 Fire in the Engine Room (5).
        if let Some(room) = state.map.rooms.get_mut(&5) {
            room.hazards.push(HazardType::Fire);
        }
    }
}
