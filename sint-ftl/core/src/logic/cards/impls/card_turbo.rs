use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Card, CardId, CardSolution, CardType, GameState, HazardType};

pub struct TurboModeCard;

impl CardBehavior for TurboModeCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::TurboMode,
            title: "Turbo Mode".to_string(),
            description: "Advantage: 3 AP. Boom: 2 Fire in Engine, 1 in Hallway.".to_string(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::types::SystemType::Engine.as_u32()),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn on_round_end(&self, state: &mut GameState) {
        let mut triggered = false;
        for card in state.active_situations.iter_mut() {
            if card.id == CardId::TurboMode {
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
            // Explosion
            if let Some(room) = state.map.rooms.get_mut(&crate::types::SystemType::Engine.as_u32()) {
                room.hazards.push(HazardType::Fire);
                room.hazards.push(HazardType::Fire);
            }
            if let Some(room) = state.map.rooms.get_mut(&crate::types::SystemType::Hallway.as_u32()) {
                room.hazards.push(HazardType::Fire);
            }
            state
                .active_situations
                .retain(|c| c.id != CardId::TurboMode);
        }
    }

    // Advantage: 3 AP. Hook into `reset_ap` logic?
    // We don't have a hook for AP reset. We set it to 2 in `advance_phase`.
    // We should add a `modify_max_ap` hook or similar.
}
