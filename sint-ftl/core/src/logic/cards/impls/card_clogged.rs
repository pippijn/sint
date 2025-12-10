use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Action, Card, CardId, CardSolution, CardType, GameState},
    GameError,
};

pub struct CloggedPipeCard;

impl CardBehavior for CloggedPipeCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::CloggedPipe,
            title: "Clogged Pipe".to_string(),
            description: format!(
                "Kitchen ({}) is disabled.",
                crate::types::SystemType::Kitchen.as_u32()
            )
            .to_string(),
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
        if let Action::Bake = action {
            return Err(GameError::InvalidAction(
                "Clogged Pipe! Cannot Bake.".to_string(),
            ));
        }
        Ok(())
    }
}
