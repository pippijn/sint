use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Card, CardId, CardSolution, CardType, GameState};

pub struct C37Recipe;

impl CardBehavior for C37Recipe {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Recipe,
            title: "Recipe".to_string(),
            description: "Mission: Go to The Bow (2). Reward: Super Peppernuts.".to_string(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(2),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn on_round_end(&self, state: &mut GameState) {
        // Timebomb.
        // If solved (Interact at Bow), reward is given.
        // If time runs out, nothing happens (Recipe lost).
        for card in state.active_situations.iter_mut() {
            if card.id == CardId::Recipe {
                if let CardType::Timebomb { rounds_left } = &mut card.card_type {
                    if *rounds_left > 0 {
                        *rounds_left -= 1;
                    }
                }
            }
        }
        // Cleanup if empty?
        state.active_situations.retain(|c| {
            if c.id == CardId::Recipe {
                if let CardType::Timebomb { rounds_left } = c.card_type {
                    rounds_left > 0
                } else {
                    true
                }
            } else {
                true
            }
        });
    }
}
