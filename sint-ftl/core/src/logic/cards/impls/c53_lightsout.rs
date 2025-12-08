use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, GameState};

pub struct C53LightsOut;

use crate::types::{Card, CardId, CardSolution, CardType};

impl CardBehavior for C53LightsOut {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::LightsOut,
            title: "Lights Out".to_string(),
            description: "Walking costs DOUBLE (2 AP).".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(5),
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
        // Effect: Walking costs DOUBLE (2 AP).
        if let Action::Move { .. } = action {
            if base_cost > 0 {
                base_cost * 2
            } else {
                0
            }
        } else {
            base_cost
        }
    }
}
