use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
};

pub struct ListingCard;

impl CardBehavior for ListingCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Listing,
            title: "Listing Ship".to_string(),
            description: "Walking is easy (0 AP), but working is hard (2x Cost).".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Engine),
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
    ) -> Result<(), crate::GameError> {
        if let GameAction::Interact = action {
            let p = state.players.get(player_id).unwrap();
            let engine = find_room_with_system(state, SystemType::Engine);
            if Some(p.room_id) != engine {
                return Err(crate::GameError::InvalidAction(
                    "Must be in Engine to fix Listing.".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn modify_action_cost(
        &self,
        _state: &GameState,
        _player_id: &str,
        action: &GameAction,
        base_cost: i32,
    ) -> i32 {
        // Walking is FREE (0 AP). Actions cost DOUBLE (2 AP).
        match action {
            GameAction::Move { .. } => 0,
            _ => {
                if base_cost > 0 {
                    base_cost * 2
                } else {
                    0
                }
            }
        }
    }
}
