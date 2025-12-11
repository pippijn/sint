use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameState, ItemType, SystemType},
};

pub struct FluWaveCard;

impl CardBehavior for FluWaveCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::FluWave,
            title: "Flu Wave".to_string(),
            description: "Boom: Every player has only 1 AP next round.".to_string(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Sickbay),
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
            let sickbay = find_room_with_system(state, SystemType::Sickbay);
            if Some(p.room_id) != sickbay {
                return Err(crate::GameError::InvalidAction(
                    "Must be in Sickbay to cure Flu.".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn on_round_end(&self, state: &mut GameState) {
        let mut triggered = false;
        for card in state.active_situations.iter_mut() {
            if card.id == CardId::FluWave {
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
            // Keep card active (rounds_left=0) to trigger on_round_start effect next turn.
        } else {
            // Solved state is handled by Interact action removing the card.
        }
    }

    fn on_round_start(&self, state: &mut GameState) {
        let mut triggered = false;
        for card in &state.active_situations {
            if card.id == CardId::FluWave {
                if let CardType::Timebomb { rounds_left } = card.card_type {
                    if rounds_left == 0 {
                        triggered = true;
                    }
                }
            }
        }

        if triggered {
            // Apply Penalty: 1 AP
            for p in state.players.values_mut() {
                p.ap = 1;
            }
            // Remove card now that penalty is applied
            state.active_situations.retain(|c| c.id != CardId::FluWave);
        }
    }
}
