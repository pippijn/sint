use crate::logic::cards::behavior::CardBehavior;
use crate::types::{CardId, CardType, GameState};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

pub struct ManOverboardCard;

use crate::types::{Card, CardSolution};

impl CardBehavior for ManOverboardCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::ManOverboard,
            title: "Man Overboard!".to_string(),
            description: "Target Player (Random) is removed from play.".to_string(),
            card_type: CardType::Timebomb { rounds_left: 2 },
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::types::SystemType::Bow.as_u32()),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn on_activate(&self, _state: &mut GameState) {
        // Target Player (Random) is removed from play?
        // "Removed from play" implies fainted or just gone?
        // Or marked?
        // Let's mark them as Fainted for simplicity, or add a special status?
        // Rules say "Removed from play". Let's assume Fainted in Water?
        // But it's a Timebomb. "If not solved... triggers".
        // Wait, "Effect: Target Player ... is removed".
        // Does the effect happen NOW or later?
        // "Timebomb ... If not solved by then, a bad effect triggers."
        // So the effect triggers when rounds_left == 0.
        // We don't do anything on activate except maybe pick the target?
        // The card description says "Target Player (Random)". We should probably pick one and store it in card description?
        // Or pick deterministically based on seed.
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
