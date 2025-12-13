use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameState, HazardType},
};
use rand::{rngs::StdRng, Rng, SeedableRng};

pub struct AnchorLooseCard;

impl CardBehavior for AnchorLooseCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::AnchorLoose,
            title: "Anchor Loose".to_owned(),
            description: "Start of round: 1 Water token on random spot.".to_owned(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: None, // Any room
                ap_cost: 1,
                item_cost: None,
                required_players: 2,
            }),
            affected_player: None,
        }
    }

    fn on_round_end(&self, state: &mut GameState) {
        // Start of every round (handled here as end of previous round + 1).
        // Place 1 Water token on random spot.
        let mut rng = StdRng::seed_from_u64(state.rng_seed);

        let room_keys: Vec<u32> = state.map.rooms.keys().collect();
        if !room_keys.is_empty() {
            let idx = rng.random_range(0..room_keys.len());
            let target = room_keys[idx];
            state.rng_seed = rng.random();

            if let Some(room) = state.map.rooms.get_mut(&target) {
                room.hazards.push(HazardType::Water);
            }
        }
    }
}
