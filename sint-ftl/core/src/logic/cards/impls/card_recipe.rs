use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSentiment, CardSolution, CardType, GameState, ItemType, SystemType},
};

pub struct RecipeCard;

impl CardBehavior for RecipeCard {
    fn get_sentiment(&self) -> CardSentiment {
        CardSentiment::Positive
    }

    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Recipe,
            title: "Recipe".to_owned(),
            description: "Mission: Go to The Bow. Reward: Super Peppernuts (Ignores Inv Limit)."
                .to_owned(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![].into(),
            solution: Some(CardSolution {
                target_system: Some(SystemType::Bow),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
            affected_player: None,
        }
    }

    fn on_solved(&self, state: &mut GameState) {
        for p in state.players.values_mut() {
            p.inventory.push(ItemType::Peppernut);
            p.inventory.push(ItemType::Peppernut);
        }
        state.chat_log.push(crate::types::ChatMessage {
            sender: "SYSTEM".to_owned(),
            text: "Recipe found! Everyone receives Super Peppernuts (2x Ammo).".to_owned(),
            timestamp: 0,
        });
    }

    fn on_trigger(&self, state: &mut GameState) {
        state.active_situations.retain(|c| c.id != CardId::Recipe);
    }
}
