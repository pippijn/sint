use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{Card, CardId, CardSolution, CardType, GameState, SystemType},
};

pub struct OverheatingCard;

impl CardBehavior for OverheatingCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Overheating,
            title: "Overheating".to_string(),
            description: "End turn in Engine -> Lose 1 AP next round.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Engine),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn on_round_start(&self, state: &mut GameState) {
        if let Some(engine_id) = find_room_with_system(state, SystemType::Engine) {
            for p in state.players.values_mut() {
                if p.room_id == engine_id {
                    if p.ap > 0 {
                        p.ap -= 1;
                    }
                }
            }
        }
    }
}
