use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, GameState};
use crate::GameError;

pub struct AnchorStuckCard;

use crate::types::{Card, CardId, CardSolution, CardType};

impl CardBehavior for AnchorStuckCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::AnchorStuck,
            title: "Anchor Stuck".to_string(),
            description: "Evasion action (Engine) is disabled.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::logic::ROOM_BOW),
                ap_cost: 1,
                item_cost: None,
                required_players: 3,
            }),
        }
    }

    fn validate_action(
        &self,
        _state: &GameState,
        _player_id: &str,
        action: &Action,
    ) -> Result<(), GameError> {
        if let Action::EvasiveManeuvers = action {
            return Err(GameError::InvalidAction(
                "Anchor Stuck! Cannot use Evasive Maneuvers.".to_string(),
            ));
        }
        Ok(())
    }
}
