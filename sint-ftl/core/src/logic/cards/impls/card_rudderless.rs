use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
};

pub struct RudderlessCard;

impl CardBehavior for RudderlessCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Rudderless,
            title: "Rudderless".to_string(),
            description: "Hard Hits. Enemy damage tokens +1.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Bridge),
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
    ) -> Result<(), crate::GameError> {
        if let GameAction::Interact = action {
            let p = state.players.get(player_id).unwrap();
            let bridge = find_room_with_system(state, SystemType::Bridge);
            if Some(p.room_id) != bridge {
                return Err(crate::GameError::InvalidAction(
                    "Must be in Bridge to fix Rudder.".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn get_hazard_modifier(&self, _state: &GameState) -> u32 {
        1
    }
}
