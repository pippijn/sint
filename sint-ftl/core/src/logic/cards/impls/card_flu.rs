use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameState, ItemType, SystemType},
};

pub struct FluWaveCard;

impl CardBehavior for FluWaveCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::FluWave,
            title: "Flu Wave".to_owned(),
            description: "Boom: Every player has only 1 AP next round.".to_owned(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Sickbay),
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
            if card.id == CardId::FluWave
                && let CardType::Timebomb { rounds_left } = card.card_type
                && rounds_left == 0
            {
                triggered = true;
            }
        }

        if triggered {
            // Apply Penalty: 1 AP
            for p in state.players.values_mut() {
                p.ap = 1;
            }
            // Remove card now that penalty is applied
            state.active_situations.retain(|c| c.id != CardId::FluWave);
        }
    }
}
