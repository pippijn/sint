use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
    GameError,
};

pub struct SugarRushCard;

impl CardBehavior for SugarRushCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::SugarRush,
            title: "Sugar Rush".to_string(),
            description: "Move 1 room extra for free. Cannons prohibited.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Kitchen),
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
        action: &GameAction,
    ) -> Result<(), GameError> {
        if let GameAction::Interact = action {
            let p = state.players.get(player_id).unwrap();
            let kitchen = find_room_with_system(state, SystemType::Kitchen);
            if Some(p.room_id) != kitchen {
                return Err(crate::GameError::InvalidAction(
                    "Must be in Kitchen to calm Sugar Rush.".to_string(),
                ));
            }
        }

        if let GameAction::Shoot = action {
            return Err(GameError::InvalidAction(
                "Sugar Rush! Too shaky to shoot.".to_string(),
            ));
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
            0
        } else {
            base_cost
        }
    }
}
