use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
};

pub struct SlipperyDeckCard;

impl CardBehavior for SlipperyDeckCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::SlipperyDeck,
            title: "Slippery Deck".to_owned(),
            description: "Soap everywhere. Move costs 0 AP, but Actions cost +1 AP.".to_owned(),
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
        match action {
            GameAction::Move { .. } => 0,
            _ => {
                if base_cost > 0 {
                    base_cost + 1
                } else {
                    0
                }
            }
        }
    }
}
