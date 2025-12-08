use crate::logic::cards::behavior::CardBehavior;
use crate::types::GameState;

pub struct C51FalseNote;

use crate::types::{Card, CardId, CardType};

impl CardBehavior for C51FalseNote {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::FalseNote,
            title: "False Note".to_string(),
            description: "Everyone in Cannons (8) flees to Hallway (7).".to_string(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        // Effect: Everyone in Cannons (8) flees to Hallway (7).
        for p in state.players.values_mut() {
            if p.room_id == 8 {
                p.room_id = 7;
            }
        }
    }
}
