use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Action, Card, CardId, CardSolution, CardType, GameState},
};

pub struct SlipperyDeckCard;

impl CardBehavior for SlipperyDeckCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::SlipperyDeck,
            title: "Slippery Deck".to_string(),
            description: "Soap everywhere. Move costs 0 AP, but Actions cost +1 AP.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::types::SystemType::Engine.as_u32()),
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
        action: &Action,
        base_cost: i32,
    ) -> i32 {
        match action {
            Action::Move { .. } => 0, // Moves are free
            _ => {
                if base_cost > 0 {
                    base_cost + 1
                } else {
                    0
                }
            } // Actions cost +1 (unless free)
        }
    }
}
