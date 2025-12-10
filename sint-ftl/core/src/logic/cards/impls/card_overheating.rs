use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameState},
};

pub struct OverheatingCard;

impl CardBehavior for OverheatingCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Overheating,
            title: "Overheating".to_string(),
            description: format!(
                "End turn in Engine ({}) -> Lose 1 AP next round.",
                crate::types::SystemType::Engine.as_u32()
            )
            .to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::types::SystemType::Engine.as_u32()),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn on_round_start(&self, state: &mut GameState) {
        // Effect: Players who ended turn in Engine lose 1 AP.
        for p in state.players.values_mut() {
            if p.room_id == crate::types::SystemType::Engine.as_u32() {
                // Mark them?
                // We'll just reduce AP and see.
                if p.ap > 0 {
                    p.ap -= 1;
                }
            }
        }
    }
}
