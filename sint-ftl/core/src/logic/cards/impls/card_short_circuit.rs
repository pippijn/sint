use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardType, GameState, HazardType, SystemType},
};

pub struct ShortCircuitCard;

impl CardBehavior for ShortCircuitCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::ShortCircuit,
            title: "Short Circuit".to_owned(),
            description: "Spawn 1 Fire in the Engine Room.".to_owned(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        if let Some(engine_id) = find_room_with_system(state, SystemType::Engine) {
            if let Some(room) = state.map.rooms.get_mut(&engine_id) {
                room.hazards.push(HazardType::Fire);
            }
        }
    }
}
