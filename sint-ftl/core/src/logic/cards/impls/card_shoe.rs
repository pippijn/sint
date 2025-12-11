use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameState, ItemType, SystemType},
};

pub struct ShoeSettingCard;

impl CardBehavior for ShoeSettingCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::ShoeSetting,
            title: "Shoe Setting".to_string(),
            description: "Boom: All players lose their next turn.".to_string(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Engine),
                ap_cost: 1,
                item_cost: Some(ItemType::Peppernut),
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
                    "Must be in Engine to fix Shoe Setting.".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn on_round_end(&self, state: &mut GameState) {
        let mut triggered = false;
        for card in state.active_situations.iter_mut() {
            if card.id == CardId::ShoeSetting {
                if let CardType::Timebomb { rounds_left } = &mut card.card_type {
                    if *rounds_left > 0 {
                        *rounds_left -= 1;
                        if *rounds_left == 0 {
                            triggered = true;
                        }
                    }
                }
            }
        }

        if triggered {
            // Boom: All players lose their next turn.
        }
    }

    fn on_round_start(&self, state: &mut GameState) {
        let mut triggered = false;
        for card in &state.active_situations {
            if card.id == CardId::ShoeSetting {
                if let CardType::Timebomb { rounds_left } = card.card_type {
                    if rounds_left == 0 {
                        triggered = true;
                    }
                }
            }
        }

        if triggered {
            for p in state.players.values_mut() {
                p.ap = 0;
            }
            state
                .active_situations
                .retain(|c| c.id != CardId::ShoeSetting);
        }
    }
}
