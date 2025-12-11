use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
};

pub struct StickyFloorCard;

impl CardBehavior for StickyFloorCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::StickyFloor,
            title: "Sticky Floor".to_owned(),
            description: "Moving into the Kitchen costs +1 AP.".to_owned(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Kitchen),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
            affected_player: None,
        }
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
