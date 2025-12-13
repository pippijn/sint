use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameState, SystemType},
};

pub struct StrongHeadwindCard;

impl CardBehavior for StrongHeadwindCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::StrongHeadwind,
            title: "Strong Headwind".to_owned(),
            description: "Cannons are inaccurate. Hit Threshold is 5+.".to_owned(),
            card_type: CardType::Situation,
            options: vec![].into(),
            solution: Some(CardSolution {
                target_system: Some(SystemType::Bridge),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
            affected_player: None,
        }
    }

    fn get_hit_threshold(&self, _state: &GameState) -> u32 {
        5
    }
}
