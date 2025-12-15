use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSentiment, CardType, GameState, ItemType},
};

pub struct PeppernutRainCard;

impl CardBehavior for PeppernutRainCard {
    fn get_sentiment(&self) -> CardSentiment {
        CardSentiment::Positive
    }

    fn get_struct(&self) -> Card {
        Card {
            id: CardId::PeppernutRain,
            title: "Peppernut Rain".to_owned(),
            description: "+2 Peppernuts dropped in every occupied room.".to_owned(),
            card_type: CardType::Flash,
            options: vec![].into(),
            solution: None,
            affected_player: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        // Effect: +2 Peppernuts dropped in every occupied room.
        let occupied_rooms = state
            .players
            .values()
            .map(|p| p.room_id)
            .collect::<crate::small_map::SmallSet<_>>();

        for rid in occupied_rooms {
            if let Some(room) = state.map.rooms.get_mut(&rid) {
                room.add_item(ItemType::Peppernut);
                room.add_item(ItemType::Peppernut);
            }
        }
    }
}
