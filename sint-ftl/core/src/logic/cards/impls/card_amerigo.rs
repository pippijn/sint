use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameState, ItemType, SystemType},
};

pub struct AmerigoCard;

impl CardBehavior for AmerigoCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Amerigo,
            title: "Amerigo".to_string(),
            description: "Hungry Horse in Storage. Eats 1 Peppernut per round.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Storage),
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
            // Check if player is in Storage
            let p = state.players.get(player_id).unwrap();
            let storage_room = find_room_with_system(state, SystemType::Storage);
            
            if Some(p.room_id) != storage_room {
                 return Err(crate::GameError::InvalidAction(
                    "Amerigo is in Storage. Go there to shoo him.".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn on_round_end(&self, state: &mut GameState) {
        // Eats 1 peppernut from Storage room items
        if let Some(room_id) = find_room_with_system(state, SystemType::Storage) {
            if let Some(room) = state.map.rooms.get_mut(&room_id) {
                if let Some(idx) = room.items.iter().position(|i| *i == ItemType::Peppernut) {
                    room.items.remove(idx);
                }
            }
        }
    }
}
