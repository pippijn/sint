use crate::{
    GameError,
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
};

pub struct CloggedPipeCard;

impl CardBehavior for CloggedPipeCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::CloggedPipe,
            title: "Clogged Pipe".to_owned(),
            description: "Kitchen is disabled.".to_owned(),
            card_type: CardType::Situation,
            options: vec![].into(),
            solution: Some(CardSolution {
                target_system: Some(SystemType::Kitchen),
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
        if let GameAction::Bake = action {
            return Err(GameError::InvalidAction(
                "Clogged Pipe! Cannot Bake.".to_owned(),
            ));
        }
        Ok(())
    }
}
