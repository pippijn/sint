use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, GameState, ItemType};
use crate::GameError;

pub struct SeagullAttackCard;

use crate::types::{Card, CardId, CardSolution, CardType};

impl CardBehavior for SeagullAttackCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::SeagullAttack,
            title: "Seagull Attack".to_string(),
            description: "Birds attacking ammo. Cannot Move while holding Peppernuts.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::types::SystemType::Bow.as_u32()),
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
        action: &Action,
    ) -> Result<(), GameError> {
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

    fn check_resolution(
        &self,
        state: &mut GameState,
        player_id: &str,
        action: &Action,
    ) -> Result<(), GameError> {
        self.validate_action(state, player_id, action)
    }
}
