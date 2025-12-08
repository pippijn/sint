use crate::logic::cards::behavior::CardBehavior;
use crate::types::GameState;

pub struct C44WheelClamp;

use crate::types::{Card, CardId, CardType, CardSolution};

impl CardBehavior for C44WheelClamp {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::WheelClamp,
            title: "Wheel Clamp".to_string(),
            description: "Ship turns. Players shift 1 Room to the right.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution { room_id: Some(9), ap_cost: 1, item_cost: None, required_players: 1 }),
        }
    }

    fn on_round_end(&self, state: &mut GameState) {
        // Effect: Ship turns. Players shift 1 Room to the right (Clockwise).
        // Map: 2 -> 3 -> 4 -> 5 -> 6 -> 8 -> 9 -> 10 -> 11 -> 2?
        // Let's assume a cycle.
        let cycle = [2, 3, 4, 5, 6, 8, 9, 10, 11];

        for p in state.players.values_mut() {
            if let Some(pos) = cycle.iter().position(|&r| r == p.room_id) {
                let next_idx = (pos + 1) % cycle.len();
                p.room_id = cycle[next_idx];
            } else if p.room_id == 7 {
                // Hallway -> Random? Or stay?
                // Text says "Every player shifts".
                // Let's keep Hallway as Hallway.
            }
        }
    }
}
