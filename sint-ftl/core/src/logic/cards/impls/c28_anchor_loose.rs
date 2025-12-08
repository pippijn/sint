use crate::logic::cards::behavior::CardBehavior;
use crate::types::{GameState, HazardType};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub struct C28AnchorLoose;

impl CardBehavior for C28AnchorLoose {
    fn on_round_end(&self, state: &mut GameState) {
        // Start of every round (handled here as end of previous round + 1).
        // Place 1 Water token on random spot.
        let mut rng = StdRng::seed_from_u64(state.rng_seed);
        // Roll 2d6 (2-12)? Rooms are 2-11.
        let target = rng.gen_range(1..=6) + rng.gen_range(1..=6);
        state.rng_seed = rng.gen();

        if let Some(room) = state.map.rooms.get_mut(&target) {
            room.hazards.push(HazardType::Water);
        }
    }
}
