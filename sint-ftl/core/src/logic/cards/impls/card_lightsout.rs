use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
};

pub struct LightsOutCard;

impl CardBehavior for LightsOutCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::LightsOut,
            title: "Lights Out".to_owned(),
            description: "Walking costs DOUBLE (2 AP).".to_owned(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Engine),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
            affected_player: None,
        }
    }

    fn modify_action_cost(
        &self,
        _state: &GameState,
        _player_id: &str,
        action: &GameAction,
        base_cost: i32,
    ) -> i32 {
        if let GameAction::Move { .. } = action {
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
