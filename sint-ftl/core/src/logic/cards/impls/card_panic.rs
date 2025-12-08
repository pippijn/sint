use crate::logic::cards::behavior::CardBehavior;
use crate::types::GameState;

pub struct PanicCard;

use crate::types::{Card, CardId, CardType};

impl CardBehavior for PanicCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Panic,
            title: "Panic!".to_string(),
            description: "Everyone runs away screaming to Dormitory (3).".to_string(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        // All players move to Dormitory (3).
        for p in state.players.values_mut() {
            p.room_id = crate::types::SystemType::Dormitory.as_u32();
        }
    }
}
