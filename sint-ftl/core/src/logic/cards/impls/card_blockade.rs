use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, GameState};
use crate::GameError;

pub struct BlockadeCard;

use crate::types::{Card, CardId, CardSolution, CardType};

impl CardBehavior for BlockadeCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Blockade,
            title: "Blockade".to_string(),
            description: format!(
                "Door to Cannons ({}) is closed.",
                crate::logic::ROOM_CANNONS
            )
            .to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::logic::ROOM_HALLWAY),
                ap_cost: 1,
                item_cost: None,
                required_players: 2,
            }),
        }
    }

    fn validate_action(
        &self,
        state: &GameState,
        player_id: &str,
        action: &Action,
    ) -> Result<(), GameError> {
        // Door to Cannons (8) is closed.
        // No one can enter or exit.
        if let Action::Move { to_room } = action {
            if *to_room == crate::logic::ROOM_CANNONS {
                return Err(GameError::InvalidAction(
                    format!(
                        "Blockade! Cannot enter Room {}.",
                        crate::logic::ROOM_CANNONS
                    )
                    .to_string(),
                ));
            }
            if let Some(p) = state.players.get(player_id) {
                if p.room_id == crate::logic::ROOM_CANNONS {
                    return Err(GameError::InvalidAction(
                        format!("Blockade! Cannot exit Room {}.", crate::logic::ROOM_CANNONS)
                            .to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}
