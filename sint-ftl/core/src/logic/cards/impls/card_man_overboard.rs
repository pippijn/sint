use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameState, SystemType},
};
use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};

pub struct ManOverboardCard;

impl CardBehavior for ManOverboardCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::ManOverboard,
            title: "Man Overboard!".to_owned(),
            description: "Target Player (Random) is removed from play.".to_owned(),
            card_type: CardType::Timebomb { rounds_left: 2 },
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Bow),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
            affected_player: None,
        }
    }

    fn on_activate(&self, _state: &mut GameState) {
        // Effect triggers when timebomb reaches 0 (in on_round_end).
    }

    fn on_round_end(&self, state: &mut GameState) {
        let mut triggered = false;

        for card in state.active_situations.iter_mut() {
            if card.id == CardId::ManOverboard {
                if let CardType::Timebomb { rounds_left } = &mut card.card_type {
                    if *rounds_left > 0 {
                        *rounds_left -= 1;
                        if *rounds_left == 0 {
                            triggered = true;
                        }
                    }
                }
            }
        }

        if triggered {
            // Remove random player
            let mut rng = StdRng::seed_from_u64(state.rng_seed);
            let pids: Vec<String> = state.players.keys().cloned().collect();
            if let Some(victim) = pids.choose(&mut rng) {
                // Remove player
                state.players.remove(victim);
            }
            state.rng_seed = rng.gen();

            state
                .active_situations
                .retain(|c| c.id != CardId::ManOverboard);
        }
    }
}
