use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Card, CardId, CardSolution, CardType, GameState};

pub struct TheBookCard;

impl CardBehavior for TheBookCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::TheBook,
            title: "The Book".to_string(),
            description: "Mission: Storage (11) -> Bridge (9). Reward: Skip Enemy Attack."
                .to_string(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(9),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn on_round_end(&self, state: &mut GameState) {
        // "Reward" triggered by solving?
        // Solution logic is "Interact". We don't track if it was solved here.
        // But if the card is REMOVED by solution, the effect triggers?
        // Card text: "Mission: Get book... REWARD: Enemy skips NEXT ATTACK."
        // Solution logic in `resolution.rs` removes the card.
        // We need to hook into removal?
        // Or `on_round_end` just ticks timebomb.
        // If solved, `resolution.rs` removes it.
        // We need a way to apply reward upon resolution.
        // But `CardBehavior` doesn't have `on_solved`.

        // Timebomb tick
        for card in state.active_situations.iter_mut() {
            if card.id == CardId::TheBook {
                if let CardType::Timebomb { rounds_left } = &mut card.card_type {
                    if *rounds_left > 0 {
                        *rounds_left -= 1;
                    }
                }
            }
        }
        state.active_situations.retain(|c| {
            if c.id == CardId::TheBook {
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
