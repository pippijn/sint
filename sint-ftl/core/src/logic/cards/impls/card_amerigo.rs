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
        // Effect: Ship Split. Can NOT go through the Hallway (crate::types::SystemType::Hallway.as_u32()).
        // Means you cannot Move TO crate::types::SystemType::Hallway.as_u32() or Move FROM crate::types::SystemType::Hallway.as_u32()?
        // "Can NOT go through the Hallway".
        // Usually implies entering or exiting is blocked?
        // Or "from one side to the other".
        // Let's block ENTRY and EXIT for simplicity, effectively isolating it (unless you are in it).
        // But the solution requires ENTERING it.
        // "Solution: Enter the Hallway (7) and feed Amerigo".
        // So you CAN enter. You just can't cross?
        // "Walk around via The Bow".
        // If I am in 6 (Kitchen), neighbor is 7. If I can't enter 7, I'm stuck?
        // Maybe it blocks "Through" traffic?
        // Pathfinding handles "Through".
        // If we block ENTRY, we block Solution.
        // Let's say: Movement COST is infinite? No.
        // Let's block "Move { to_room: 7 }" UNLESS it is the Solution action?
        // But Solution is "Interact".
        // So we can block "Move to 7".
        // BUT "Solution: Enter Hallway". This implies you MUST enter.
        // Maybe the restriction is: You cannot Move *from* 7?
        // If I move 6 -> 7. Next turn 7 -> 8.
        // If I block 7->Any, I can enter but not leave.
        // Let's implement: "Cannot Move from Room 7 to anywhere else"?
        // "Walk around via The Bow (2)".
        // Let's block all Moves FROM 7.
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
