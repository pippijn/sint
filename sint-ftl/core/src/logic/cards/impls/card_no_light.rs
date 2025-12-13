use crate::{
    GameError,
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
};

pub struct NoLightCard;

impl CardBehavior for NoLightCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::NoLight,
            title: "No Light?".to_owned(),
            description: "Shooting prohibited. The cannons don't work.".to_owned(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Cargo),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
            affected_player: None,
        }
    }

    fn validate_action(
        &self,
        _state: &GameState,
        _player_id: &str,
        action: &GameAction,
    ) -> Result<(), GameError> {
        if let GameAction::Shoot = action {
            return Err(GameError::InvalidAction(
                "No Light! Cannons can't aim.".to_owned(),
            ));
        }
        Ok(())
    }
}
