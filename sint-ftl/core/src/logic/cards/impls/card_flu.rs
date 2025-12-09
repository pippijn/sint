use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Card, CardId, CardSolution, CardType, GameState, ItemType};

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
                room_id: Some(crate::types::SystemType::Sickbay.as_u32()),
                ap_cost: 1,
                item_cost: Some(ItemType::Peppernut),
                required_players: 1,
            }),
        }
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
