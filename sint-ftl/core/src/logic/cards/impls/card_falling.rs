use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardType, GameState, HazardType, ItemType, SystemType},
};

pub struct FallingGiftCard;

impl CardBehavior for FallingGiftCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::FallingGift,
            title: "Falling Gift".to_owned(),
            description: "Leak in Cargo. +2 Peppernuts in Cargo.".to_owned(),
            card_type: CardType::Flash,
            options: vec![].into(),
            solution: None,
            affected_player: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        if let Some(cargo_id) = find_room_with_system(state, SystemType::Cargo)
            && let Some(room) = state.map.rooms.get_mut(&cargo_id)
        {
            room.add_hazard(HazardType::Water);
            room.add_item(ItemType::Peppernut);
            room.add_item(ItemType::Peppernut);
        }
    }
}
