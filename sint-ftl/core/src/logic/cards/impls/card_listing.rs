use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
};

pub struct ListingCard;

impl CardBehavior for ListingCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Listing,
            title: "Listing Ship".to_string(),
            description: "Walking is easy (0 AP), but working is hard (2x Cost).".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Engine),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn modify_action_cost(
        &self,
        _state: &GameState,
        _player_id: &str,
        action: &GameAction,
        base_cost: i32,
    ) -> i32 {
        // Walking is FREE (0 AP). Actions cost DOUBLE (2 AP).
        match action {
            GameAction::Move { .. } => 0,
            _ => {
                if base_cost > 0 {
                    base_cost * 2
                } else {
                    0
                }
            }
        }
    }
}
