use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Action, Card, CardId, CardSolution, CardType, GameState},
    GameError,
};

pub struct NoLightCard;

impl CardBehavior for NoLightCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::NoLight,
            title: "No Light?".to_string(),
            description: "Shooting prohibited. The cannons don't work.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::types::SystemType::Cargo.as_u32()),
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
        if let Action::Shoot = action {
            return Err(GameError::InvalidAction(
                "No Light! Cannons can't aim.".to_string(),
            ));
        }
        Ok(())
    }
}
