use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameState, SystemType},
};

pub struct OverheatingCard;

impl CardBehavior for OverheatingCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Overheating,
            title: "Overheating".to_string(),
            description: "End turn in Engine -> Lose 1 AP next round.".to_string(),
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
        action: &crate::types::GameAction,
    ) -> Result<(), crate::GameError> {
        if let crate::types::GameAction::Interact = action {
            let p = state.players.get(player_id).unwrap();
            let engine = find_room_with_system(state, SystemType::Engine);
            if Some(p.room_id) != engine {
                return Err(crate::GameError::InvalidAction(
                    "Must be in Engine to fix Overheating.".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn on_round_start(&self, state: &mut GameState) {
        if let Some(engine_id) = find_room_with_system(state, SystemType::Engine) {
            for p in state.players.values_mut() {
                if p.room_id == engine_id {
                    if p.ap > 0 {
                        p.ap -= 1;
                    }
                }
            }
        }
    }
}
