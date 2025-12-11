use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameState, SystemType},
};

pub struct MutinyCard;

impl CardBehavior for MutinyCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Mutiny,
            title: "Mutiny?".to_owned(),
            description: "If not solved, Game Over (or -10 Hull).".to_owned(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Bridge),
                ap_cost: 1,
                item_cost: None,
                required_players: 2,
            }),
        }
    }

    fn on_round_end(&self, state: &mut GameState) {
        let mut triggered_damage = false;

        for card in state.active_situations.iter_mut() {
            if card.id == CardId::Mutiny {
                if let CardType::Timebomb { rounds_left } = &mut card.card_type {
                    if *rounds_left > 0 {
                        *rounds_left -= 1;
                        if *rounds_left == 0 {
                            triggered_damage = true;
                        }
                    }
                }
            }
        }

        if triggered_damage {
            state.hull_integrity -= 10;
            state.active_situations.retain(|c| c.id != CardId::Mutiny);
        }
    }
}
