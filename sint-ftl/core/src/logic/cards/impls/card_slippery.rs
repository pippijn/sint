use crate::{
    logic::cards::behavior::CardBehavior,
    types::{
        Card, CardId, CardSentiment, CardSolution, CardType, GameAction, GameState, SystemType,
    },
};

pub struct SlipperyDeckCard;

impl CardBehavior for SlipperyDeckCard {
    fn get_sentiment(&self) -> CardSentiment {
        CardSentiment::Neutral
    }

    fn get_struct(&self) -> Card {
        Card {
            id: CardId::SlipperyDeck,
            title: "Slippery Deck".to_owned(),
            description:
                "Soap everywhere. Moving into Hallways costs 0 AP. Other actions cost +1 AP. Lasts 3 rounds."
                    .to_owned(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Engine),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
            affected_player: None,
        }
    }

    fn on_round_end(&self, state: &mut GameState) {
        let mut expired = false;
        for card in state.active_situations.iter_mut() {
            if card.id == CardId::SlipperyDeck
                && let CardType::Timebomb { rounds_left } = &mut card.card_type
                && *rounds_left > 0
            {
                *rounds_left -= 1;
                if *rounds_left == 0 {
                    expired = true;
                }
            }
        }
        if expired {
            state
                .active_situations
                .retain(|c| c.id != CardId::SlipperyDeck);
        }
    }

    fn modify_action_cost(
        &self,
        state: &GameState,
        _player_id: &str,
        action: &GameAction,
        base_cost: i32,
    ) -> i32 {
        match action {
            // Moving into a Hallway (Room with no system) is free
            GameAction::Move { to_room } => {
                if let Some(room) = state.map.rooms.get(to_room)
                    && room.system.is_none()
                {
                    return 0;
                }
                base_cost
            }
            _ => {
                if base_cost > 0 {
                    base_cost + 1
                } else {
                    0
                }
            }
        }
    }
}
