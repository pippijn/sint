use crate::{
    GameError,
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
};

pub struct MonsterDoughCard;

impl CardBehavior for MonsterDoughCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::MonsterDough,
            title: "Monster Dough".to_owned(),
            description: "Boom: Kitchen is unusable.".to_owned(),
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

    fn validate_action(
        &self,
        state: &GameState,
        _player_id: &str,
        action: &GameAction,
    ) -> Result<(), GameError> {
        // If triggered (rounds_left == 0)
        // Block actions in Kitchen.
        if let GameAction::Bake = action {
            let triggered = state.active_situations.iter().any(|c| {
                c.id == CardId::MonsterDough
                    && matches!(c.card_type, CardType::Timebomb { rounds_left: 0 })
            });
            if triggered {
                return Err(GameError::InvalidAction(
                    "Monster Dough! Kitchen blocked.".to_owned(),
                ));
            }
        }
        Ok(())
    }
}
