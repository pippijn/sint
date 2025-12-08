use crate::logic::cards::behavior::CardBehavior;
use crate::types::{GameState, HazardType};

pub struct BigLeakCard;

use crate::types::{Card, CardId, CardSolution, CardType};

impl CardBehavior for BigLeakCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::BigLeak,
            title: "The Big Leak".to_string(),
            description: format!(
                "Flooding. Start of round: 1 Water in Hallway ({}).",
                crate::types::SystemType::Hallway.as_u32()
            )
            .to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::types::SystemType::Hallway.as_u32()),
                ap_cost: 1,
                item_cost: None,
                required_players: 2,
            }),
        }
    }

    fn on_round_end(&self, state: &mut GameState) {
        // Automatically 1 Water in Central Hallway (7).
        if let Some(room) = state
            .map
            .rooms
            .get_mut(&crate::types::SystemType::Hallway.as_u32())
        {
            room.hazards.push(HazardType::Water);
        }
    }
}
