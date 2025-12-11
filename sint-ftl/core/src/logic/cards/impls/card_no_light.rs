use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
    GameError,
};

pub struct NoLightCard;

impl CardBehavior for NoLightCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::NoLight,
            title: "No Light?".to_string(),
            description: "Shooting prohibited. The cannons don't work.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Cargo),
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
            let cargo = find_room_with_system(state, SystemType::Cargo);
            if Some(p.room_id) != cargo {
                return Err(crate::GameError::InvalidAction(
                    "Must be in Cargo to fix Lights.".to_string(),
                ));
            }
        }

        if let GameAction::Shoot = action {
            return Err(GameError::InvalidAction(
                "No Light! Cannons can't aim.".to_string(),
            ));
        }
        Ok(())
    }
}
