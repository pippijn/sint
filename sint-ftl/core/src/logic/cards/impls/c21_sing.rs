use crate::logic::cards::behavior::CardBehavior;
use crate::types::{GameState, HazardType};

pub struct C21SingASong;

impl CardBehavior for C21SingASong {
    fn on_activate(&self, state: &mut GameState) {
        // Remove 2 Water OR 2 Fire.
        // Heuristic: Prioritize Fire (Damage) then Water.
        let mut removed_count = 0;
        let limit = 2;

        // Remove Fire
        for room in state.map.rooms.values_mut() {
            while removed_count < limit {
                if let Some(idx) = room.hazards.iter().position(|h| *h == HazardType::Fire) {
                    room.hazards.remove(idx);
                    removed_count += 1;
                } else {
                    break;
                }
            }
        }

        // If still quota, remove Water
        for room in state.map.rooms.values_mut() {
            while removed_count < limit {
                if let Some(idx) = room.hazards.iter().position(|h| *h == HazardType::Water) {
                    room.hazards.remove(idx);
                    removed_count += 1;
                } else {
                    break;
                }
            }
        }
    }
}
