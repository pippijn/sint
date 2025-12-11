use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameState, ItemType, SystemType},
};

pub struct MicePlagueCard;

impl CardBehavior for MicePlagueCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::MicePlague,
            title: "Mice Plague".to_string(),
            description: "At end of round, lose 2 Peppernuts from Storage.".to_string(),
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
            let p = state.players.get(player_id).unwrap();
            let storage = find_room_with_system(state, SystemType::Storage);
            if Some(p.room_id) != storage {
                return Err(crate::GameError::InvalidAction(
                    "Must be in Storage to clear Mice.".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn on_round_end(&self, state: &mut GameState) {
        if let Some(storage_id) = find_room_with_system(state, SystemType::Storage) {
            if let Some(room) = state.map.rooms.get_mut(&storage_id) {
                // Remove up to 2 peppernuts
                let mut removed = 0;
                room.items.retain(|item| {
                    if *item == ItemType::Peppernut && removed < 2 {
                        removed += 1;
                        false // Drop
                    } else {
                        true // Keep
                    }
                });
            }
        }
    }
}
