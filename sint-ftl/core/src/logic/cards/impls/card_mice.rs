use crate::logic::cards::behavior::CardBehavior;
use crate::types::{GameState, ItemType};

pub struct MicePlagueCard;

use crate::types::{Card, CardId, CardSolution, CardType};

impl CardBehavior for MicePlagueCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::MicePlague,
            title: "Mice Plague".to_string(),
            description: format!(
                "At end of round, lose 2 Peppernuts from Storage ({}).",
                crate::types::SystemType::Storage.as_u32()
            )
            .to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::types::SystemType::Storage.as_u32()),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn on_round_end(&self, state: &mut GameState) {
        // Effect: At end of round, lose 2 Peppernuts from Storage (11).
        if let Some(room) = state
            .map
            .rooms
            .get_mut(&crate::types::SystemType::Storage.as_u32())
        {
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
