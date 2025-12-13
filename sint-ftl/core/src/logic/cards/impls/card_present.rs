use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSentiment, CardType, GameState, ItemType},
};

pub struct PresentCard;

impl CardBehavior for PresentCard {
    fn get_sentiment(&self) -> CardSentiment {
        CardSentiment::Positive
    }

    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Present,
            title: "Present".to_owned(),
            description: "Choose your gift: Repair 3 Tokens.".to_owned(),
            card_type: CardType::Flash,
            options: vec![],
            solution: None,
            affected_player: None,
        }
    }

    fn on_activate(&self, state: &mut GameState) {
        // Option B: Repair 3 Tokens.
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
