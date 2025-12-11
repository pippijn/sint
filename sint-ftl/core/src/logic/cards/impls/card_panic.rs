use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardType, GameState, SystemType},
};

pub struct PanicCard;

impl CardBehavior for PanicCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Panic,
            title: "Panic!".to_owned(),
            description: "Everyone runs away screaming to Dormitory.".to_owned(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        if let Some(dorm_id) = find_room_with_system(state, SystemType::Dormitory) {
            for p in state.players.values_mut() {
                p.room_id = dorm_id;
            }
        }
    }
}
