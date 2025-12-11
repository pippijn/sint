use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameState, SystemType},
};

pub struct MutinyCard;

impl CardBehavior for MutinyCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Mutiny,
            title: "Mutiny?".to_string(),
            description: "If not solved, Game Over (or -10 Hull).".to_string(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Bridge),
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
        action: &crate::types::GameAction,
    ) -> Result<(), crate::GameError> {
        if let crate::types::GameAction::Interact = action {
            let p = state.players.get(player_id).unwrap();
            let bridge = find_room_with_system(state, SystemType::Bridge);
            if Some(p.room_id) != bridge {
                return Err(crate::GameError::InvalidAction(
                    "Must be in Bridge to stop Mutiny.".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn on_round_end(&self, state: &mut GameState) {
        let mut triggered_damage = false;

        for card in state.active_situations.iter_mut() {
            if card.id == CardId::Mutiny {
                if let CardType::Timebomb { rounds_left } = &mut card.card_type {
                    if *rounds_left > 0 {
                        *rounds_left -= 1;
                        if *rounds_left == 0 {
                            triggered_damage = true;
                        }
                    }
                }
            }
        }

        if triggered_damage {
            state.hull_integrity -= 10;
            state.active_situations.retain(|c| c.id != CardId::Mutiny);
        }
    }
}
