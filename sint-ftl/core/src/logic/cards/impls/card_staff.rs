use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameState, PlayerStatus},
};

pub struct TheStaffCard;

impl CardBehavior for TheStaffCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::TheStaff,
            title: "The Staff".to_string(),
            description: format!(
                "Mission: {} ({}) -> {} ({}) . Reward: Magical Recovery.",
                "Dormitory",
                crate::types::SystemType::Dormitory.as_u32(),
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
        for p in state.players.values_mut() {
            p.hp = 3;
            p.status.retain(|s| *s != PlayerStatus::Fainted);
        }
        state.chat_log.push(crate::types::ChatMessage {
            sender: "SYSTEM".to_string(),
            text: "The Staff is activated! The crew is fully healed via Magical Recovery."
                .to_string(),
            timestamp: 0,
        });
    }

    fn on_round_end(&self, state: &mut GameState) {
        for card in state.active_situations.iter_mut() {
            if card.id == CardId::TheStaff {
                if let CardType::Timebomb { rounds_left } = &mut card.card_type {
                    if *rounds_left > 0 {
                        *rounds_left -= 1;
                    }
                }
            }
        }
        state.active_situations.retain(|c| {
            if c.id == CardId::TheStaff {
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
