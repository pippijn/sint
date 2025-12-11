use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
};

pub struct RudderlessCard;

impl CardBehavior for RudderlessCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Rudderless,
            title: "Rudderless".to_owned(),
            description: "Hard Hits. Enemy damage tokens +1.".to_owned(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Bridge),
                ap_cost: 1,
                item_cost: None,
                required_players: 2,
            }),
            affected_player: None,
        }
    }

    fn validate_action(
        &self,
        _state: &GameState,
        _player_id: &str,
        _action: &GameAction,
    ) -> Result<(), crate::GameError> {
        Ok(())
    }

    fn get_hazard_modifier(&self, _state: &GameState) -> u32 {
        1
    }
}
