use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameState, SystemType},
};

pub struct WheelClampCard;

impl CardBehavior for WheelClampCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::WheelClamp,
            title: "Wheel Clamp".to_owned(),
            description: "Ship turns. Players shift to (Room ID + 1).".to_owned(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Bridge),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn on_round_end(&self, state: &mut GameState) {
        let total_rooms = state.map.rooms.len() as u32;
        if total_rooms == 0 {
            return;
        }

        for p in state.players.values_mut() {
            p.room_id = (p.room_id + 1) % total_rooms;
        }
    }
}
