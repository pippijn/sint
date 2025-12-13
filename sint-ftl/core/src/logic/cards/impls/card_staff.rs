use crate::{
    logic::cards::behavior::CardBehavior,
    types::{
        Card, CardId, CardSentiment, CardSolution, CardType, GameState, PlayerStatus, SystemType,
    },
};

pub struct TheStaffCard;

impl CardBehavior for TheStaffCard {
    fn get_sentiment(&self) -> CardSentiment {
        CardSentiment::Positive
    }

    fn get_struct(&self) -> Card {
        Card {
            id: CardId::TheStaff,
            title: "The Staff".to_owned(),
            description: "Mission: Dormitory -> Bridge. Reward: Magical Recovery.".to_owned(),
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
        for p in state.players.values_mut() {
            p.hp = 3;
            p.status.retain(|s| *s != PlayerStatus::Fainted);
        }
        state.chat_log.push(crate::types::ChatMessage {
            sender: "SYSTEM".to_owned(),
            text: "The Staff is activated! The crew is fully healed via Magical Recovery."
                .to_owned(),
            timestamp: 0,
        });
    }

    fn on_trigger(&self, state: &mut GameState) {
        state.active_situations.retain(|c| c.id != CardId::TheStaff);
    }
}
