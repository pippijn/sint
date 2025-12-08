use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, GameState};
use crate::GameError;

pub struct JammedCannonCard;

use crate::types::{Card, CardId, CardSolution, CardType, ItemType};

impl CardBehavior for JammedCannonCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::JammedCannon,
            title: "Jammed Cannon".to_string(),
            description: format!("Cannons ({}) are disabled.", crate::logic::ROOM_CANNONS)
                .to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::logic::ROOM_CANNONS),
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
        action: &Action,
    ) -> Result<(), GameError> {
        if let Action::Shoot = action {
            return Err(GameError::InvalidAction(
                "Cannon Jammed! Cannot Shoot.".to_string(),
            ));
        }
        Ok(())
    }
}
