use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
    GameError,
};

pub struct StaticNoiseCard;

impl CardBehavior for StaticNoiseCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::StaticNoise,
            title: "Static Noise".to_string(),
            description: "Radio interference. Chat restricted to Emoji Only.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: None, // Enforced dynamically
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn validate_action(
        &self,
        state: &GameState,
        player_id: &str,
        action: &GameAction,
    ) -> Result<(), GameError> {
        if let GameAction::Interact = action {
            let p = state.players.get(player_id).unwrap();
            let bridge = find_room_with_system(state, SystemType::Bridge);
            if Some(p.room_id) != bridge {
                return Err(GameError::InvalidAction(
                    "Must be in Bridge to fix Static Noise.".to_string(),
                ));
            }
        }

        if let GameAction::Chat { message } = action {
            // Check for non-emoji characters (simplified: alphabetic)
            if message.chars().any(|c| c.is_alphabetic()) {
                return Err(GameError::Silenced);
            }
        }
        Ok(())
    }
}
