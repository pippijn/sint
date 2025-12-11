use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameState, SystemType},
};

pub struct TheBookCard;

impl CardBehavior for TheBookCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::TheBook,
            title: "The Book".to_string(),
            description: "Mission: Storage -> Bridge. Reward: Skip Enemy Attack.".to_string(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Bridge),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn validate_action(
        &self,
        state: &GameState,
        player_id: &str,
        action: &crate::types::GameAction,
    ) -> Result<(), crate::GameError> {
        if let crate::types::GameAction::Interact = action {
            let p = state.players.get(player_id).unwrap();
            let bridge = find_room_with_system(state, SystemType::Bridge);
            if Some(p.room_id) != bridge {
                return Err(crate::GameError::InvalidAction(
                    "Mission complete at Bridge.".to_string(),
                ));
            }
        }
        Ok(())
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
