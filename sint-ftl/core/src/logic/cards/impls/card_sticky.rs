use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, Card, CardId, CardSolution, CardType, GameState};

pub struct StickyFloorCard;

impl CardBehavior for StickyFloorCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::StickyFloor,
            title: "Sticky Floor".to_string(),
            description: "Moving into the Hallway costs +1 AP.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::types::SystemType::Kitchen.as_u32()),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn modify_action_cost(
        &self,
        _state: &GameState,
        _player_id: &str,
        action: &Action,
        current_cost: i32,
    ) -> i32 {
        if let Action::Move { to_room } = action {
            if *to_room == crate::types::SystemType::Hallway.as_u32() {
                return current_cost + 1;
            }
        }
        current_cost
    }
}
