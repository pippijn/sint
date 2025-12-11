use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameState, ItemType, SystemType},
};

pub struct MicePlagueCard;

impl CardBehavior for MicePlagueCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::MicePlague,
            title: "Mice Plague".to_owned(),
            description: "At end of round, lose 2 Peppernuts from Storage.".to_owned(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Storage),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
            affected_player: None,
        }
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
