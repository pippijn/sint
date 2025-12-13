use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameState, HazardType, SystemType},
};

pub struct WeirdGiftsCard;

impl CardBehavior for WeirdGiftsCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::WeirdGifts,
            title: "Weird Gifts".to_owned(),
            description: "Boom: 3 Fire in Cargo, 1 Fire in Sickbay.".to_owned(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Cargo),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
            affected_player: None,
        }
    }

    fn on_round_end(&self, state: &mut GameState) {
        let mut triggered = false;
        for card in state.active_situations.iter_mut() {
            if card.id == CardId::WeirdGifts
                && let CardType::Timebomb { rounds_left } = &mut card.card_type
                && *rounds_left > 0
            {
                *rounds_left -= 1;
                if *rounds_left == 0 {
                    triggered = true;
                }
            }
        }

        if triggered {
            if let Some(cargo_id) = find_room_with_system(state, SystemType::Cargo)
                && let Some(room) = state.map.rooms.get_mut(&cargo_id)
            {
                for _ in 0..3 {
                    room.hazards.push(HazardType::Fire);
                }
            }
            if let Some(sickbay_id) = find_room_with_system(state, SystemType::Sickbay)
                && let Some(room) = state.map.rooms.get_mut(&sickbay_id)
            {
                room.hazards.push(HazardType::Fire);
            }
            state
                .active_situations
                .retain(|c| c.id != CardId::WeirdGifts);
        }
    }
}
