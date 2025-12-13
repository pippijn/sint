use crate::{
    logic::cards::behavior::CardBehavior,
    types::{
        Card, CardId, CardSentiment, CardSolution, CardType, ChatMessage, GameState, SystemType,
    },
};

pub struct TheBookCard;

impl CardBehavior for TheBookCard {
    fn get_sentiment(&self) -> CardSentiment {
        CardSentiment::Positive
    }

    fn get_struct(&self) -> Card {
        Card {
            id: CardId::TheBook,
            title: "The Book".to_owned(),
            description: "Mission: Storage -> Bridge. Reward: Skip Enemy Attack.".to_owned(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Bridge),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
            affected_player: None,
        }
    }

    fn on_solved(&self, state: &mut GameState) {
        state.enemy.next_attack = None;
        state.chat_log.push(ChatMessage {
            sender: "SYSTEM".to_owned(),
            text: "The Book is recovered! The enemy is confused and skips their attack.".to_owned(),
            timestamp: 0,
        });
    }

    fn on_trigger(&self, state: &mut GameState) {
        state.active_situations.retain(|c| c.id != CardId::TheBook);
    }
}
