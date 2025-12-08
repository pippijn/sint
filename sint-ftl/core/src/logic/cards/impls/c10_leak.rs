use crate::logic::cards::behavior::CardBehavior;
use crate::types::{GameState, HazardType};

pub struct C10Leak;

impl CardBehavior for C10Leak {
    fn on_activate(&self, state: &mut GameState) {
        // Effect: Spawn 1 Water in the Cargo Room (4).
        if let Some(room) = state.map.rooms.get_mut(&4) {
            room.hazards.push(HazardType::Water);
        }
    }
}
