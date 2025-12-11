use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardType, GameState, HazardType, SystemType},
};

pub struct LeakCard;

impl CardBehavior for LeakCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Leak,
            title: "Leak!".to_owned(),
            description: "Spawn 1 Water in the Cargo Room.".to_owned(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        if let Some(cargo_id) = find_room_with_system(state, SystemType::Cargo) {
            if let Some(room) = state.map.rooms.get_mut(&cargo_id) {
                room.hazards.push(HazardType::Water);
            }
        }
    }
}
