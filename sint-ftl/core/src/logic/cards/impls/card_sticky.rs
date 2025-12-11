use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
};

pub struct StickyFloorCard;

impl CardBehavior for StickyFloorCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::StickyFloor,
            title: "Sticky Floor".to_string(),
            description: "Moving into the Kitchen costs +1 AP.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Kitchen),
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
    ) -> Result<(), crate::GameError> {
        if let GameAction::Interact = action {
            // Check if player is in Kitchen (Solution location)
            // Handlers usually check if solution.room_id (now target_system) matches.
            // But we can double check here to be safe or rely on updated InteractHandler.
            let p = state.players.get(player_id).unwrap();
            let kitchen = find_room_with_system(state, SystemType::Kitchen);
            if Some(p.room_id) != kitchen {
                return Err(crate::GameError::InvalidAction(
                    "Must be in Kitchen to clean Sticky Floor.".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn modify_action_cost(
        &self,
        state: &GameState,
        _player_id: &str,
        action: &GameAction,
        current_cost: i32,
    ) -> i32 {
        if let GameAction::Move { to_room } = action {
            if let Some(kitchen_id) = find_room_with_system(state, SystemType::Kitchen) {
                if *to_room == kitchen_id {
                    return current_cost + 1;
                }
            }
        }
        current_cost
    }
}
