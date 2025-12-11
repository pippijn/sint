use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameState, HazardType, SystemType},
};

pub struct BigLeakCard;

impl CardBehavior for BigLeakCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::BigLeak,
            title: "The Big Leak".to_string(),
            description: "Flooding. Start of round: 1 Water in Cargo.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Cargo),
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
        action: &crate::types::GameAction,
    ) -> Result<(), crate::GameError> {
        if let crate::types::GameAction::Interact = action {
            let p = state.players.get(player_id).unwrap();
            let cargo_room = find_room_with_system(state, SystemType::Cargo);
            if Some(p.room_id) != cargo_room {
                return Err(crate::GameError::InvalidAction(
                    "Big Leak is in Cargo.".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn on_round_end(&self, state: &mut GameState) {
        if let Some(room_id) = find_room_with_system(state, SystemType::Cargo) {
            if let Some(room) = state.map.rooms.get_mut(&room_id) {
                room.hazards.push(HazardType::Water);
            }
        }
    }
}
