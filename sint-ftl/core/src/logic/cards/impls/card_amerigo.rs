use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, ItemType},
    GameError,
};

pub struct AmerigoCard;

impl CardBehavior for AmerigoCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Amerigo,
            title: "Amerigo".to_string(),
            description: format!(
                "Ship Split. Can't cross Hallway ({}).",
                crate::types::SystemType::Hallway.as_u32()
            )
            .to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::types::SystemType::Hallway.as_u32()),
                ap_cost: 1,
                item_cost: Some(ItemType::Peppernut),
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
        // Implementation: Amerigo prevents leaving the Hallway (Room 7) once inside,
        // forcing players to find another path (e.g. via Bow).
        if let GameAction::Move { .. } = action {
            if let Some(p) = state.players.get(player_id) {
                if p.room_id == crate::types::SystemType::Hallway.as_u32() {
                    return Err(GameError::InvalidAction(
                        "Amerigo blocks the way! You cannot leave the Hallway.".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}
