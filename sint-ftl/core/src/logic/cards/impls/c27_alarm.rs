use crate::logic::cards::behavior::CardBehavior;

pub struct C27WailingAlarm;

use crate::types::{Card, CardId, CardSolution, CardType};

impl CardBehavior for C27WailingAlarm {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::WailingAlarm,
            title: "Wailing Alarm".to_string(),
            description: "No Bonuses. Special items and skills don't work.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(7),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }
}
