use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameState, SystemType},
};
use rand::{Rng, SeedableRng, prelude::IndexedRandom, rngs::StdRng};

pub struct ManOverboardCard;

impl CardBehavior for ManOverboardCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::ManOverboard,
            title: "Man Overboard!".to_owned(),
            description: "Target Player (Random) is removed from play.".to_owned(),
            card_type: CardType::Timebomb { rounds_left: 2 },
            options: vec![].into(),
            solution: Some(CardSolution {
                target_system: Some(SystemType::Bow),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
            affected_player: None,
        }
    }

    fn on_trigger(&self, state: &mut GameState) {
        // Remove random player
        let mut rng = StdRng::seed_from_u64(state.rng_seed);
        let pids: Vec<String> = state.players.keys().cloned().collect();
        if let Some(victim) = pids.choose(&mut rng) {
            // Remove player
            state.players.remove(victim);
        }
        state.rng_seed = rng.random();

        state
            .active_situations
            .retain(|c| c.id != CardId::ManOverboard);
    }
}
