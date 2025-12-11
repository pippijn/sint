use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameState, ItemType, SystemType},
};

pub struct RecipeCard;

impl CardBehavior for RecipeCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Recipe,
            title: "Recipe".to_string(),
            description: "Mission: Go to The Bow. Reward: Super Peppernuts.".to_string(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Bow),
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
            let bow = find_room_with_system(state, SystemType::Bow);
            if Some(p.room_id) != bow {
                return Err(crate::GameError::InvalidAction(
                    "Recipe is in The Bow.".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn on_solved(&self, state: &mut GameState) {
        for p in state.players.values_mut() {
            p.inventory.push(ItemType::Peppernut);
            p.inventory.push(ItemType::Peppernut);
        }
        state.chat_log.push(crate::types::ChatMessage {
            sender: "SYSTEM".to_string(),
            text: "Recipe found! Everyone receives Super Peppernuts (2x Ammo).".to_string(),
            timestamp: 0,
        });
    }

    fn on_round_end(&self, state: &mut GameState) {
        for card in state.active_situations.iter_mut() {
            if card.id == CardId::Recipe {
                if let CardType::Timebomb { rounds_left } = &mut card.card_type {
                    if *rounds_left > 0 {
                        *rounds_left -= 1;
                    }
                }
            }
        }
        state.active_situations.retain(|c| {
            if c.id == CardId::Recipe {
                if let CardType::Timebomb { rounds_left } = c.card_type {
                    rounds_left > 0
                } else {
                    true
                }
            } else {
                true
            }
        });
    }
}
