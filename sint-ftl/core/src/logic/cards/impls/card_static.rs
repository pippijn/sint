use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
    GameError,
};

pub struct StaticNoiseCard;

impl CardBehavior for StaticNoiseCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::StaticNoise,
            title: "Static Noise".to_owned(),
            description: "Radio interference. Chat restricted to Emoji Only.".to_owned(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Bridge),
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
        if let GameAction::Chat { message } = action {
            // Check for non-emoji characters (simplified: alphabetic)
            if message.chars().any(|c| c.is_alphabetic()) {
                return Err(GameError::Silenced);
            }
        }
        Ok(())
    }
}
