use crate::logic::cards::behavior::CardBehavior;
use crate::types::{GameState, HazardType};

pub struct C30BigLeak;

impl CardBehavior for C30BigLeak {
    fn on_round_end(&self, state: &mut GameState) {
        // Automatically 1 Water in Central Hallway (7).
        if let Some(room) = state.map.rooms.get_mut(&7) {
            room.hazards.push(HazardType::Water);
        }
    }
}
