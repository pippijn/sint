use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Card, CardId, CardSolution, CardType, GameState};

pub struct GoldenNutCard;

impl CardBehavior for GoldenNutCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::GoldenNut,
            title: "Golden Nut".to_string(),
            description: "Mission: Go to Storage (11). Reward: Auto Hit.".to_string(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(11),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn on_round_end(&self, state: &mut GameState) {
        for card in state.active_situations.iter_mut() {
            if card.id == CardId::GoldenNut {
                if let CardType::Timebomb { rounds_left } = &mut card.card_type {
                    if *rounds_left > 0 {
                        *rounds_left -= 1;
                    }
                }
            }
        }
        state.active_situations.retain(|c| {
            if c.id == CardId::GoldenNut {
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
