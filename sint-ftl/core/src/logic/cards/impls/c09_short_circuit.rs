use crate::logic::cards::behavior::CardBehavior;
use crate::types::{GameState, HazardType};

pub struct C09ShortCircuit;

impl CardBehavior for C09ShortCircuit {
    fn on_activate(&self, state: &mut GameState) {
        // Effect: Spawn 1 Fire in the Engine Room (5).
        if let Some(room) = state.map.rooms.get_mut(&5) {
            room.hazards.push(HazardType::Fire);
        }
    }
}
