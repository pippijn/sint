use crate::logic::cards::behavior::CardBehavior;

pub struct FogBankCard;

use crate::types::{Card, CardId, CardSolution, CardType};

impl CardBehavior for FogBankCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::FogBank,
            title: "Fog Bank".to_string(),
            description: "Cannot see Enemy Intent (Telegraph disabled).".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::types::SystemType::Bow.as_u32()),
                ap_cost: 2,
                item_cost: None,
                required_players: 1,
            }),
        }
    }
}
