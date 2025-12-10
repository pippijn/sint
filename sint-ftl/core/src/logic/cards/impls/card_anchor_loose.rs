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
            title: "Anchor Loose".to_string(),
            description: "Start of round: 1 Water token on random spot.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(2),
                ap_cost: 1,
                item_cost: None,
                required_players: 2,
            }),
        }
    }

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
