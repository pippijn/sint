use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Card, CardId, CardSolution, CardType, GameState, ItemType};

pub struct C35Stowaway;

impl CardBehavior for C35Stowaway {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Stowaway,
            title: "The Stowaway".to_string(),
            description: "Boom: All players lose all peppernuts.".to_string(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(3),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn on_round_end(&self, state: &mut GameState) {
        let mut triggered = false;
        for card in state.active_situations.iter_mut() {
            if card.id == CardId::Stowaway {
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
            // Theft: Lose all peppernuts
            for p in state.players.values_mut() {
                p.inventory.retain(|i| *i != ItemType::Peppernut);
            }
            state.active_situations.retain(|c| c.id != CardId::Stowaway);
        }
    }
}
