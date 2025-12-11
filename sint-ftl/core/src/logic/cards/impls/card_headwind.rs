use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameState, SystemType},
};

pub struct StrongHeadwindCard;

impl CardBehavior for StrongHeadwindCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::StrongHeadwind,
            title: "Strong Headwind".to_string(),
            description: "Cannons are inaccurate. Hit Threshold is 5+.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Bridge),
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
        action: &crate::types::GameAction,
    ) -> Result<(), crate::GameError> {
        if let crate::types::GameAction::Interact = action {
            let p = state.players.get(player_id).unwrap();
            let bridge = find_room_with_system(state, SystemType::Bridge);
            if Some(p.room_id) != bridge {
                return Err(crate::GameError::InvalidAction(
                    "Must be in Bridge to navigate Headwind.".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn get_hit_threshold(&self, _state: &GameState) -> u32 {
        5
    }
}
