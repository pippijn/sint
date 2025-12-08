use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, GameState};
use crate::GameError;

pub struct C18CloggedPipe;

use crate::types::{Card, CardId, CardSolution, CardType};

impl CardBehavior for C18CloggedPipe {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::CloggedPipe,
            title: "Clogged Pipe".to_string(),
            description: "Kitchen (6) is disabled.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(6),
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
        if let Action::Bake = action {
            return Err(GameError::InvalidAction(
                "Clogged Pipe! Cannot Bake.".to_string(),
            ));
        }
        Ok(())
    }
}
