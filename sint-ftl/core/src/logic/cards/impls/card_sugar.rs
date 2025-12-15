use crate::{
    GameError,
    logic::cards::behavior::CardBehavior,
    types::{
        Card, CardId, CardSentiment, CardSolution, CardType, GameAction, GameState, SystemType,
    },
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
            description: "Move to any room for only 1 AP. Cannons prohibited. Lasts 3 rounds."
                .to_owned(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![].into(),
            solution: Some(CardSolution {
                target_system: Some(SystemType::Kitchen),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
            affected_player: None,
        }
    }

    fn on_trigger(&self, state: &mut GameState) {
        state
            .active_situations
            .retain(|c| c.id != CardId::SugarRush);
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

    fn can_reach(
        &self,
        _state: &GameState,
        _player_id: &str,
        _to_room: crate::types::RoomId,
    ) -> bool {
        true
    }
}
