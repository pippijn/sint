use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameState, HazardType, SystemType},
};

pub struct BigLeakCard;

impl CardBehavior for BigLeakCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::BigLeak,
            title: "The Big Leak".to_owned(),
            description: "Flooding. Start of round: 1 Water in Cargo.".to_owned(),
            card_type: CardType::Situation,
            options: vec![].into(),
            solution: Some(CardSolution {
                target_system: Some(SystemType::Cargo),
                ap_cost: 1,
                item_cost: None,
                required_players: 2,
            }),
            affected_player: None,
        }
    }

    fn on_round_start(&self, state: &mut GameState) {
        if let Some(room_id) = find_room_with_system(state, SystemType::Cargo)
            && let Some(room) = state.map.rooms.get_mut(&room_id)
        {
            room.add_hazard(HazardType::Water);
        }
    }
}
