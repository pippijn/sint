use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, GameState};
use crate::GameError;

pub struct C02StaticNoise;

use crate::types::{Card, CardId, CardSolution, CardType};

impl CardBehavior for C02StaticNoise {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::StaticNoise,
            title: "Static Noise".to_string(),
            description: "Radio interference. Chat restricted to Emoji Only.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(9),
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
        if let Action::Chat { message } = action {
            // Check for non-emoji characters (simplified: alphabetic)
            if message.chars().any(|c| c.is_alphabetic()) {
                return Err(GameError::Silenced);
            }
        }
        Ok(())
    }
}
