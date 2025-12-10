use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState},
    GameError,
};

pub struct SeasickCard;

impl CardBehavior for SeasickCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Seasick,
            title: "Seasick".to_string(),
            description: "Nauseous. You may EITHER Walk OR do Actions (not both).".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::types::SystemType::Kitchen.as_u32()),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn validate_action(
        &self,
        _state: &GameState,
        _player_id: &str,
        _action: &GameAction,
    ) -> Result<(), GameError> {
        // Effect: You may EITHER Walk OR do Actions (not both).
        // Note: Strict validation skipped for now (requires tracking intent across batch).
        Ok(())
    }
}
