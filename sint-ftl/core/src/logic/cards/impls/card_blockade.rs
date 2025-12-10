use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState},
    GameError,
};

pub struct BlockadeCard;

impl CardBehavior for BlockadeCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Blockade,
            title: "Blockade".to_string(),
            description: format!(
                "Door to Cannons ({}) is closed.",
                crate::types::SystemType::Cannons.as_u32()
            )
            .to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::types::SystemType::Hallway.as_u32()),
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
        action: &GameAction,
    ) -> Result<(), GameError> {
        // Door to Cannons (8) is closed.
        // No one can enter or exit.
        if let GameAction::Move { to_room } = action {
            if *to_room == crate::types::SystemType::Cannons.as_u32() {
                return Err(GameError::InvalidAction(
                    format!(
                        "Blockade! Cannot enter Room {}.",
                        crate::types::SystemType::Cannons.as_u32()
                    )
                    .to_string(),
                ));
            }
            if let Some(p) = state.players.get(player_id) {
                if p.room_id == crate::types::SystemType::Cannons.as_u32() {
                    return Err(GameError::InvalidAction(
                        format!(
                            "Blockade! Cannot exit Room {}.",
                            crate::types::SystemType::Cannons.as_u32()
                        )
                        .to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}
