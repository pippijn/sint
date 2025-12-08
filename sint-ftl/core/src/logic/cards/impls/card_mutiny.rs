use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Card, CardId, CardSolution, CardType, GameState};

pub struct MutinyCard;

impl CardBehavior for MutinyCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Mutiny,
            title: "Mutiny?".to_string(),
            description: "If not solved, Game Over (or -10 Hull).".to_string(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::logic::ROOM_BRIDGE),
                ap_cost: 1,
                item_cost: None,
                required_players: 2,
            }),
        }
    }

    fn on_round_end(&self, state: &mut GameState) {
        // Find self in active situations to decrement timer
        // Effect: If not solved by end of countdown (rounds_left == 0), Game Over (or -10 Hull).

        let mut triggered_damage = false;

        // This is tricky because we are modifying the vector we are finding in?
        // But `state` is mutable.

        for card in state.active_situations.iter_mut() {
            if card.id == CardId::Mutiny {
                if let CardType::Timebomb { rounds_left } = &mut card.card_type {
                    if *rounds_left > 0 {
                        *rounds_left -= 1;
                        if *rounds_left == 0 {
                            triggered_damage = true;
                        }
                    }
                }
            }
        }

        if triggered_damage {
            state.hull_integrity -= 10;
            // Remove card? Logic usually removes resolved cards.
            // But here it exploded. We should probably remove it.
            // Or let `resolve_hazards` or similar cleanup?
            // Let's remove it.
            state.active_situations.retain(|c| c.id != CardId::Mutiny);
        }
    }
}
