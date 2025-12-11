use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameState, SystemType},
};

pub struct WheelClampCard;

impl CardBehavior for WheelClampCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::WheelClamp,
            title: "Wheel Clamp".to_string(),
            description: "Ship turns. Players shift to (Room ID + 1).".to_string(),
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
                    "Must be in Bridge to release Wheel Clamp.".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn on_round_end(&self, state: &mut GameState) {
        let total_rooms = state.map.rooms.len() as u32;
        if total_rooms == 0 {
            return;
        }

        for p in state.players.values_mut() {
            p.room_id = (p.room_id + 1) % total_rooms;
        }
    }
}
