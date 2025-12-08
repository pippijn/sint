use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, GameState};
use crate::GameError;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub struct C40StickyFloor;

use crate::types::{Card, CardId, CardSolution, CardType};

impl CardBehavior for C40StickyFloor {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::StickyFloor,
            title: "Sticky Floor".to_string(),
            description: "Leaving a room? Roll 1-3: Stuck.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(6),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn check_resolution(
        &self,
        state: &mut GameState,
        _player_id: &str,
        action: &Action,
    ) -> Result<(), GameError> {
        if let Action::Move { .. } = action {
            // Roll die 1-6. 1-3 Stuck.
            let mut rng = StdRng::seed_from_u64(state.rng_seed);
            let roll: u32 = rng.gen_range(1..=6);
            state.rng_seed = rng.gen();

            if roll <= 3 {
                return Err(GameError::InvalidAction(
                    "Stuck on sticky floor!".to_string(),
                ));
            }
        }
        Ok(())
    }
}
