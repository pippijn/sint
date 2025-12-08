use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, GameState, ItemType};
use crate::GameError;

pub struct C03SeagullAttack;

impl CardBehavior for C03SeagullAttack {
    fn validate_action(&self, state: &GameState, player_id: &str, action: &Action) -> Result<(), GameError> {
        if let Action::Move { .. } = action {
            if let Some(player) = state.players.get(player_id) {
                if player.inventory.contains(&ItemType::Peppernut) {
                    return Err(GameError::InvalidAction(
                        "Cannot move while holding Peppernuts (Seagull Attack)".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn check_resolution(&self, state: &mut GameState, player_id: &str, action: &Action) -> Result<(), GameError> {
        self.validate_action(state, player_id, action)
    }
}