use crate::{
    logic::cards::behavior::CardBehavior,
    types::{
        Card, CardId, CardSentiment, CardSolution, CardType, ChatMessage, EnemyState, GamePhase,
        GameState, SystemType,
    },
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
            options: vec![].into(),
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
        state.chat_log.push(ChatMessage {
            sender: "SYSTEM".to_owned(),
            text: "Golden Nut used! 1 Damage dealt to the Enemy.".to_owned(),
            timestamp: 0,
        });

        // Check for Boss Death
        if state.enemy.hp <= 0 {
            if state.boss_level >= crate::logic::MAX_BOSS_LEVEL - 1 {
                state.phase = GamePhase::Victory;
                state.chat_log.push(ChatMessage {
                    sender: "SYSTEM".to_owned(),
                    text: "VICTORY! All bosses defeated!".to_owned(),
                    timestamp: 0,
                });
            } else {
                // Mark defeated to trigger rest round in advance_phase
                state.enemy.state = EnemyState::Defeated;
                state.chat_log.push(ChatMessage {
                    sender: "SYSTEM".to_owned(),
                    text: format!("{} Defeated! Taking a breather...", state.enemy.name),
                    timestamp: 0,
                });
            }
        }
    }

    fn on_trigger(&self, state: &mut GameState) {
        state
            .active_situations
            .retain(|c| c.id != CardId::GoldenNut);
    }
}
