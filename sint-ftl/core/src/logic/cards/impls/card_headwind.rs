use crate::logic::cards::behavior::CardBehavior;
use crate::types::GameState;

pub struct StrongHeadwindCard;

use crate::types::{Card, CardId, CardSolution, CardType};

impl CardBehavior for StrongHeadwindCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::StrongHeadwind,
            title: "Strong Headwind".to_string(),
            description: "Cannons are inaccurate. Hit Threshold is 5+.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::types::SystemType::Bridge.as_u32()),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn get_hit_threshold(&self, _state: &GameState) -> u32 {
        5
    }
}
