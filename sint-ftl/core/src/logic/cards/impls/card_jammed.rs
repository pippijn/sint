use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, ItemType, SystemType},
    GameError,
};

pub struct JammedCannonCard;

impl CardBehavior for JammedCannonCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::JammedCannon,
            title: "Jammed Cannon".to_string(),
            description: "Cannons are disabled.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Cannons),
                ap_cost: 1,
                item_cost: Some(ItemType::Peppernut),
                required_players: 1,
            }),
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
                "Cannon Jammed! Cannot Shoot.".to_string(),
            ));
        }
        Ok(())
    }
}
