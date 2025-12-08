use crate::logic::cards::behavior::CardBehavior;
use crate::types::{GameState, ItemType};

pub struct C41Present;

use crate::types::{Card, CardId, CardType};

impl CardBehavior for C41Present {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Present,
            title: "Present".to_string(),
            description: "Choose your gift: Repair 3 Tokens.".to_string(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        // Choose your gift.
        // Simplified: Everyone gets full AP (Option A) AND Repair 3 (Option B).
        // Let's be generous for the holidays.

        // Option A: AP refill (though usually AP is reset in Telegraph, so refill now is only useful if we are in Action phase?
        // But Flash cards are drawn in MorningReport. So AP refill is useless as AP resets later anyway.
        // Wait, "Everyone gets all AP back". If drawn in Morning, AP is already full?
        // No, AP resets in EnemyTelegraph.
        // If we set AP=2 now, it persists until reset?
        // Or "Max AP +1"?
        // Let's implement Option B: Repair 3 Tokens.

        let mut repaired = 0;
        let limit = 3;

        // Repair Fire
        for room in state.map.rooms.values_mut() {
            while repaired < limit && !room.hazards.is_empty() {
                room.hazards.pop();
                repaired += 1;
            }
        }

        // Option C: Ammo. Give everyone 1 Peppernut.
        if repaired < limit {
            for p in state.players.values_mut() {
                if p.inventory.len() < 3 {
                    p.inventory.push(ItemType::Peppernut);
                }
            }
        }
    }
}
