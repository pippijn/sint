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
            options: vec![].into(),
            solution: Some(CardSolution {
                target_system: Some(SystemType::Bridge),
                ap_cost: 1,
                item_cost: None,
                required_players: 2,
            }),
            affected_player: None,
        }
    }

    fn on_trigger(&self, state: &mut GameState) {
        state.hull_integrity -= 10;
        state.active_situations.retain(|c| c.id != CardId::Mutiny);
    }
}
