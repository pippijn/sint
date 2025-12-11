use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameState, ItemType, SystemType},
};

pub struct StowawayCard;

impl CardBehavior for StowawayCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Stowaway,
            title: "The Stowaway".to_string(),
            description: "Boom: All players lose all peppernuts.".to_string(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Dormitory),
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
            let dorm = find_room_with_system(state, SystemType::Dormitory);
            if Some(p.room_id) != dorm {
                return Err(crate::GameError::InvalidAction(
                    "Must be in Dormitory to find Stowaway.".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn on_round_end(&self, state: &mut GameState) {
        let mut triggered = false;
        for card in state.active_situations.iter_mut() {
            if card.id == CardId::Stowaway {
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
            // Theft: Lose all peppernuts
            for p in state.players.values_mut() {
                p.inventory.retain(|i| *i != ItemType::Peppernut);
            }
            state.active_situations.retain(|c| c.id != CardId::Stowaway);
        }
    }
}
