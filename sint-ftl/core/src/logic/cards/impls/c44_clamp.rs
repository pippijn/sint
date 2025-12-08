use crate::logic::cards::behavior::CardBehavior;
use crate::types::GameState;

pub struct C44WheelClamp;

impl CardBehavior for C44WheelClamp {
    fn on_round_end(&self, state: &mut GameState) {
        // Effect: Ship turns. Players shift 1 Room to the right (Clockwise).
        // Map: 2 -> 3 -> 4 -> 5 -> 6 -> 8 -> 9 -> 10 -> 11 -> 2?
        // Let's assume a cycle.
        let cycle = [2, 3, 4, 5, 6, 8, 9, 10, 11];
        
        for p in state.players.values_mut() {
            if let Some(pos) = cycle.iter().position(|&r| r == p.room_id) {
                let next_idx = (pos + 1) % cycle.len();
                p.room_id = cycle[next_idx];
            } else if p.room_id == 7 {
                // Hallway -> Random? Or stay?
                // Text says "Every player shifts".
                // Let's keep Hallway as Hallway.
            }
        }
    }
}
