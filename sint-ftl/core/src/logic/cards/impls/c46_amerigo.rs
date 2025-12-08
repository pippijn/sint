use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, GameState};
use crate::GameError;

pub struct C46Amerigo;

use crate::types::{Card, CardId, CardType, CardSolution, ItemType};

impl CardBehavior for C46Amerigo {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Amerigo,
            title: "Amerigo".to_string(),
            description: "Ship Split. Can't cross Hallway (7).".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution { room_id: Some(7), ap_cost: 1, item_cost: Some(ItemType::Peppernut), required_players: 1 }),
        }
    }

    fn validate_action(
        &self,
        state: &GameState,
        player_id: &str,
        action: &Action,
    ) -> Result<(), GameError> {
        // Effect: Ship Split. Can NOT go through the Hallway (7).
        // Means you cannot Move TO 7 or Move FROM 7?
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
        // Maybe the effect is: "Cannot Move FROM 7 to anywhere else"?
        // Or "Cannot Move TO 7"?
        // Text: "You can NOT go through the Hallway (7) from one side to the other."
        // "Want to go to the other side? Walk around."
        // This implies Room 7 is not a valid node for pathfinding?
        // But we need to enter it to solve.
        // Let's assume: You can enter 7, but you cannot EXIT 7?
        // No, then you are trapped.
        // Let's assume: It blocks movement *between* specific rooms?
        // Simplified logic: "Hallway is blocked".
        // But Solution says "Enter Hallway".
        // Maybe the restriction is: You cannot Move *from* 7?
        // If I move 6 -> 7. Next turn 7 -> 8.
        // If I block 7->Any, I can enter but not leave.
        // Let's implement: "Cannot Move from Room 7 to any room except The Bow (2)?"
        // "Walk around via The Bow (2)".
        // Let's block all Moves FROM 7.
        if let Action::Move { .. } = action {
            if let Some(p) = state.players.get(player_id) {
                if p.room_id == 7 {
                    return Err(GameError::InvalidAction(
                        "Amerigo blocks the way! You cannot leave the Hallway.".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}
