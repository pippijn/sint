use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameState},
};

pub struct TheBookCard;

impl CardBehavior for TheBookCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::TheBook,
            title: "The Book".to_string(),
            description: format!(
                "Mission: {} ({}) -> {} ({}) . Reward: Skip Enemy Attack.",
                "Storage",
                crate::types::SystemType::Storage.as_u32(),
                "Bridge",
                crate::types::SystemType::Bridge.as_u32()
            )
            .to_string(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::types::SystemType::Bridge.as_u32()),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn on_solved(&self, state: &mut GameState) {
        state.enemy.next_attack = None;
        state.chat_log.push(crate::types::ChatMessage {
            sender: "SYSTEM".to_string(),
            text: "The Book is recovered! The enemy is confused and skips their attack."
                .to_string(),
            timestamp: 0,
        });
    }

    fn on_round_end(&self, state: &mut GameState) {
        // Reward is applied when the solution action (Interact) is successfully executed.
        // Here we just handle the Timebomb countdown.

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
