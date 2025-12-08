use crate::logic::cards::behavior::CardBehavior;
use crate::types::{GameState, HazardType};

pub struct C48SilentForce;

use crate::types::{Card, CardId, CardType};

impl CardBehavior for C48SilentForce {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::SilentForce,
            title: "Silent Force".to_string(),
            description: "Remove 3 Tokens (Fire or Water) from the board.".to_string(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        // Effect: Remove 3 Tokens (Fire or Water) from the board.
        let mut removed = 0;
        let limit = 3;

        // 1. Remove Fire (Priority)
        for room in state.map.rooms.values_mut() {
            while removed < limit {
                if let Some(idx) = room.hazards.iter().position(|h| *h == HazardType::Fire) {
                    room.hazards.remove(idx);
                    removed += 1;
                } else {
                    break;
                }
            }
        }

        // 2. Remove Water
        for room in state.map.rooms.values_mut() {
            while removed < limit {
                if let Some(idx) = room.hazards.iter().position(|h| *h == HazardType::Water) {
                    room.hazards.remove(idx);
                    removed += 1;
                } else {
                    break;
                }
            }
        }
    }
}
