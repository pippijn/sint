use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
    GameError,
};

pub struct SugarRushCard;

impl CardBehavior for SugarRushCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::SugarRush,
            title: "Sugar Rush".to_owned(),
            description: "Move 5 rooms extra for free. Cannons prohibited.".to_owned(),
            card_type: CardType::Situation,
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
