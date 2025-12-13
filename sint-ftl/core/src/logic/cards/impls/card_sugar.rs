use crate::{
    logic::cards::behavior::CardBehavior,
    types::{
        Card, CardId, CardSentiment, CardSolution, CardType, GameAction, GameState, SystemType,
    },
    GameError,
};

pub struct SugarRushCard;

impl CardBehavior for SugarRushCard {
    fn get_sentiment(&self) -> CardSentiment {
        CardSentiment::Neutral
    }

    fn get_struct(&self) -> Card {
        Card {
            id: CardId::SugarRush,
            title: "Sugar Rush".to_owned(),
            description: "Move 5 rooms extra for free. Cannons prohibited. Lasts 3 rounds."
                .to_owned(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Kitchen),
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
            if card.id == CardId::SugarRush {
                if let CardType::Timebomb { rounds_left } = &mut card.card_type {
                    if *rounds_left > 0 {
                        *rounds_left -= 1;
                        if *rounds_left == 0 {
                            expired = true;
                        }
                    }
                }
            }
        }
        if expired {
            state
                .active_situations
                .retain(|c| c.id != CardId::SugarRush);
        }
    }

    fn validate_action(
        &self,
        _state: &GameState,
        _player_id: &str,
        action: &GameAction,
    ) -> Result<(), GameError> {
        if let GameAction::Shoot = action {
            return Err(GameError::InvalidAction(
                "Sugar Rush! Too shaky to shoot.".to_owned(),
            ));
        }
        Ok(())
    }

    fn modify_action_cost(
        &self,
        state: &GameState,
        player_id: &str,
        action: &GameAction,
        base_cost: i32,
    ) -> i32 {
        if let GameAction::Move { .. } = action {
            // Count how many moves are already in the queue for this player
            let moves_queued = state
                .proposal_queue
                .iter()
                .filter(|p| p.player_id == player_id && matches!(p.action, GameAction::Move { .. }))
                .count();

            if moves_queued < 5 {
                0
            } else {
                base_cost
            }
        } else {
            base_cost
        }
    }
}
