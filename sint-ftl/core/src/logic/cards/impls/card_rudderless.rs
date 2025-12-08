use crate::logic::cards::behavior::CardBehavior;

pub struct RudderlessCard;

use crate::types::{Card, CardId, CardSolution, CardType};

impl CardBehavior for RudderlessCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Rudderless,
            title: "Rudderless".to_string(),
            description: "Hard Hits. Enemy damage tokens +1.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::logic::ROOM_BRIDGE),
                ap_cost: 1,
                item_cost: None,
                required_players: 2,
            }),
        }
    }
}
