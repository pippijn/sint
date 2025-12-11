use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardType, GameState},
};
use rand::{rngs::StdRng, Rng, SeedableRng};

pub struct HighPressureCard;

impl CardBehavior for HighPressureCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::HighPressure,
            title: "High Pressure".to_owned(),
            description: "Adrenaline! Everyone moves 1 step.".to_owned(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        // Effect: All players take 1 step to a room of choice.
        // Implementation: Move to a random neighbor to simulate chaos.
        let player_ids: Vec<String> = state.players.keys().cloned().collect();

        for pid in player_ids {
            let current_room_id = state.players[&pid].room_id;
            if let Some(room) = state.map.rooms.get(&current_room_id) {
                if !room.neighbors.is_empty() {
                    let mut rng = StdRng::seed_from_u64(state.rng_seed);
                    let idx = rng.gen_range(0..room.neighbors.len());
                    let next_room = room.neighbors[idx];
                    state.rng_seed = rng.gen();

                    if let Some(p) = state.players.get_mut(&pid) {
                        p.room_id = next_room;
                    }
                }
            }
        }
    }
}
