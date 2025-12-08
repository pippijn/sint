use crate::logic::cards::behavior::CardBehavior;
use crate::types::GameState;

pub struct FalseNoteCard;

use crate::types::{Card, CardId, CardType};

impl CardBehavior for FalseNoteCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::FalseNote,
            title: "False Note".to_string(),
            description: format!(
                "Everyone in Cannons ({}) flees to Hallway ({}).",
                crate::logic::ROOM_CANNONS,
                crate::logic::ROOM_HALLWAY
            )
            .to_string(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        // Effect: Everyone in Cannons (8) flees to Hallway (7).
        for p in state.players.values_mut() {
            if p.room_id == crate::logic::ROOM_CANNONS {
                p.room_id = crate::logic::ROOM_HALLWAY;
            }
        }
    }
}
