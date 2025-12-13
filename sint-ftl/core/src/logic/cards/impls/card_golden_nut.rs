use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSentiment, CardSolution, CardType, GameState, SystemType},
};

pub struct GoldenNutCard;

impl CardBehavior for GoldenNutCard {
    fn get_sentiment(&self) -> CardSentiment {
        CardSentiment::Positive
    }

    fn get_struct(&self) -> Card {
        Card {
            id: CardId::GoldenNut,
            title: "Golden Nut".to_owned(),
            description: "Mission: Go to Storage. Reward: Auto Hit.".to_owned(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Storage),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
            affected_player: None,
        }
    }

    fn on_solved(&self, state: &mut GameState) {
        state.enemy.hp -= 1;
        state.chat_log.push(crate::types::ChatMessage {
            sender: "SYSTEM".to_owned(),
            text: "Golden Nut used! 1 Damage dealt to the Enemy.".to_owned(),
            timestamp: 0,
        });

        // Check for Boss Death (Duplicated from resolution.rs logic)
        if state.enemy.hp <= 0 {
            state.boss_level += 1;
            if state.boss_level >= crate::logic::MAX_BOSS_LEVEL {
                state.phase = crate::types::GamePhase::Victory;
                state.chat_log.push(crate::types::ChatMessage {
                    sender: "SYSTEM".to_owned(),
                    text: "VICTORY! All bosses defeated!".to_owned(),
                    timestamp: 0,
                });
            } else {
                // Spawn next boss
                state.enemy = crate::logic::get_boss(state.boss_level);
                state.chat_log.push(crate::types::ChatMessage {
                    sender: "SYSTEM".to_owned(),
                    text: format!("Enemy Defeated! approaching: {}", state.enemy.name),
                    timestamp: 0,
                });
            }
        }
    }

    fn on_round_end(&self, state: &mut GameState) {
        for card in state.active_situations.iter_mut() {
            if card.id == CardId::GoldenNut
                && let CardType::Timebomb { rounds_left } = &mut card.card_type
                && *rounds_left > 0
            {
                *rounds_left -= 1;
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
