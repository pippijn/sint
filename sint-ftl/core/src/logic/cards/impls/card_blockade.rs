use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
    GameError,
};

pub struct BlockadeCard;

impl CardBehavior for BlockadeCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Blockade,
            title: "Blockade".to_owned(),
            description: "Door to Cannons is closed.".to_owned(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: None, // Any room (usually adjacent to Cannons to solve?)
                ap_cost: 1,
                item_cost: None,
                required_players: 2,
            }),
        }
    }

    fn validate_action(
        &self,
        state: &GameState,
        player_id: &str,
        action: &GameAction,
    ) -> Result<(), GameError> {
        if let Some(cannons_id) = find_room_with_system(state, SystemType::Cannons) {
            if let GameAction::Move { to_room } = action {
                if *to_room == cannons_id {
                    return Err(GameError::InvalidAction(
                        "Blockade! Cannot enter Cannons.".to_owned(),
                    ));
                }
                if let Some(p) = state.players.get(player_id) {
                    if p.room_id == cannons_id {
                        return Err(GameError::InvalidAction(
                            "Blockade! Cannot exit Cannons.".to_owned(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}
