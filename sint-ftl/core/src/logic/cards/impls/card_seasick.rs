use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
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
                target_system: Some(SystemType::Kitchen),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn validate_action(
        &self,
        state: &GameState,
        player_id: &str,
        action: &GameAction,
    ) -> Result<(), GameError> {
        if let GameAction::Interact = action {
            let p = state.players.get(player_id).unwrap();
            let kitchen = find_room_with_system(state, SystemType::Kitchen);
            if Some(p.room_id) != kitchen {
                return Err(crate::GameError::InvalidAction(
                    "Must be in Kitchen to cure Seasick.".to_string(),
                ));
            }
        }
        Ok(())
    }
}
