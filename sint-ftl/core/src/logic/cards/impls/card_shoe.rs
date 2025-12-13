use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameState, ItemType, SystemType},
};

pub struct ShoeSettingCard;

impl CardBehavior for ShoeSettingCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::ShoeSetting,
            title: "Shoe Setting".to_owned(),
            description: "Boom: All players lose their next turn.".to_owned(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Engine),
                ap_cost: 1,
                item_cost: Some(ItemType::Peppernut),
                required_players: 1,
            }),
            affected_player: None,
        }
    }

    fn on_round_start(&self, state: &mut GameState) {
        let mut triggered = false;
        for card in &state.active_situations {
            if card.id == CardId::ShoeSetting
                && let CardType::Timebomb { rounds_left } = card.card_type
                && rounds_left == 0
            {
                triggered = true;
            }
        }

        if triggered {
            for p in state.players.values_mut() {
                p.ap = 0;
            }
            state
                .active_situations
                .retain(|c| c.id != CardId::ShoeSetting);
        }
    }
}
