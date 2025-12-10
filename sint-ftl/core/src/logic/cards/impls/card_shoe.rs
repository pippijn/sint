use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameState, ItemType},
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
                room_id: Some(crate::types::SystemType::Engine.as_u32()),
                ap_cost: 1,
                item_cost: Some(ItemType::Peppernut),
                required_players: 1,
            }),
        }
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
            // Note: We set rounds_left to 0 to signal the penalty for on_round_start.
        } else {
            // Remove if solved?
        }
    }

    // We need `on_round_start` to enforce AP loss if triggered.
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
            // Now remove it so it only affects one turn
            state
                .active_situations
                .retain(|c| c.id != CardId::ShoeSetting);
        }
    }
}
