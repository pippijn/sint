use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameState, SystemType},
};

pub struct WheelClampCard;

impl CardBehavior for WheelClampCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::WheelClamp,
            title: "Wheel Clamp".to_string(),
            description: "Ship turns. Players shift 1 Room to the right.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(SystemType::Bridge.as_u32()),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn on_round_end(&self, state: &mut GameState) {
        // Effect: Ship turns. Players shift 1 Room to the right (Clockwise).
        // Map: 2 -> 3 -> 4 -> 5 -> 6 -> 8 -> 9 -> 10 -> 11 -> 2?
        let cycle = [
            SystemType::Bow.as_u32(),
            SystemType::Dormitory.as_u32(),
            SystemType::Cargo.as_u32(),
            SystemType::Engine.as_u32(),
            SystemType::Kitchen.as_u32(),
            SystemType::Cannons.as_u32(),
            SystemType::Bridge.as_u32(),
            SystemType::Sickbay.as_u32(),
            SystemType::Storage.as_u32(),
        ];

        for p in state.players.values_mut() {
            if let Some(pos) = cycle.iter().position(|&r| r == p.room_id) {
                let next_idx = (pos + 1) % cycle.len();
                p.room_id = cycle[next_idx];
            } else if p.room_id == SystemType::Hallway.as_u32() {
                // Hallway -> Random? Or stay?
                // Text says "Every player shifts".
                // Let's keep Hallway as Hallway.
            }
        }
    }
}
