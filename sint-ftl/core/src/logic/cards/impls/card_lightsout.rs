use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
};

pub struct LightsOutCard;

impl CardBehavior for LightsOutCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::LightsOut,
            title: "Lights Out".to_string(),
            description: "Walking costs DOUBLE (2 AP).".to_string(),
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

    fn validate_action(
        &self,
        state: &GameState,
        player_id: &str,
        action: &crate::types::GameAction,
    ) -> Result<(), crate::GameError> {
        if let crate::types::GameAction::Interact = action {
            let p = state.players.get(player_id).unwrap();
            let engine = find_room_with_system(state, SystemType::Engine);
            if Some(p.room_id) != engine {
                return Err(crate::GameError::InvalidAction(
                    "Must be in Engine to fix Lights Out.".to_string(),
                ));
            }
        }
        Ok(())
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
