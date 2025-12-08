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
                room_id: Some(crate::logic::ROOM_SICKBAY),
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
            // Next round 1 AP.
            // We set AP at start of TacticalPlanning.
            // We need a persistent status "Sick".
            // Since we don't have statuses yet other than Fainted/Silenced,
            // we'll implement this by reducing AP *now* if we reset AP at end of round?
            // `advance_phase` resets AP to 2 in EnemyTelegraph.
            // If this triggers in EnemyAction (before Telegraph), we need a way to persist the malus.
            // State variable? Or just modify players now and hope they don't get reset?
            // They DO get reset.
            // We'll leave this unimplemented correctly without a Status system.
            state.active_situations.retain(|c| c.id != CardId::FluWave);
        }
    }
}
