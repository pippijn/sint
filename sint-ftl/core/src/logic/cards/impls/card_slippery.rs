use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
};

pub struct SlipperyDeckCard;

impl CardBehavior for SlipperyDeckCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::SlipperyDeck,
            title: "Slippery Deck".to_string(),
            description: "Soap everywhere. Move costs 0 AP, but Actions cost +1 AP.".to_string(),
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
                    "Must be in Engine to clean Slippery Deck.".to_string(),
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
        match action {
            GameAction::Move { .. } => 0,
            _ => {
                if base_cost > 0 {
                    base_cost + 1
                } else {
                    0
                }
            }
        }
    }
}
