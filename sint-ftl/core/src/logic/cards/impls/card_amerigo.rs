use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameState, ItemType, SystemType},
};

pub struct AmerigoCard;

impl CardBehavior for AmerigoCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Amerigo,
            title: "Amerigo".to_owned(),
            description: "Hungry Horse in Storage. Eats 1 Peppernut per round.".to_owned(),
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
