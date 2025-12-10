use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Action, Card, CardId, CardSolution, CardType, GameState},
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
                room_id: Some(crate::types::SystemType::Kitchen.as_u32()),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn validate_action(
        &self,
        _state: &GameState,
        _player_id: &str,
        action: &Action,
    ) -> Result<(), GameError> {
        // Negative: Cannons prohibited.
        if let Action::Shoot = action {
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
        action: &Action,
        base_cost: i32,
    ) -> i32 {
        // Positive: Move 1 room extra free.
        // Simplified: Move is free.
        if let Action::Move { .. } = action {
            0
        } else {
            base_cost
        }
    }
}
