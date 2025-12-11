use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
    GameError,
};

pub struct AnchorStuckCard;

impl CardBehavior for AnchorStuckCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::AnchorStuck,
            title: "Anchor Stuck".to_string(),
            description: "Evasion action (Engine) is disabled.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Bow),
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
        action: &GameAction,
    ) -> Result<(), GameError> {
        if let GameAction::EvasiveManeuvers = action {
            return Err(GameError::InvalidAction(
                "Anchor Stuck! Cannot use Evasive Maneuvers.".to_string(),
            ));
        }
        Ok(())
    }
}
