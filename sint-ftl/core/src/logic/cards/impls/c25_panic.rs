use crate::logic::cards::behavior::CardBehavior;
use crate::types::GameState;

pub struct C25Panic;

impl CardBehavior for C25Panic {
    fn on_activate(&self, state: &mut GameState) {
        // All players move to Dormitory (3).
        for p in state.players.values_mut() {
            p.room_id = 3;
        }
    }
}
