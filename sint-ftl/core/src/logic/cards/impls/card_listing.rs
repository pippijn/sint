use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
};

pub struct ListingCard;

impl CardBehavior for ListingCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Listing,
            title: "Listing Ship".to_owned(),
            description: "Gravity is weird. +5 AP/Round. Move is 1 AP. Others 2x Cost.".to_owned(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Engine),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
            affected_player: None,
        }
    }

    fn on_round_start(&self, state: &mut GameState) {
        for p in state.players.values_mut() {
            p.ap += 5;
        }
    }

    fn modify_action_cost(
        &self,
        _state: &GameState,
        _player_id: &str,
        action: &GameAction,
        base_cost: i32,
    ) -> i32 {
        match action {
            GameAction::Move { .. } => base_cost, // Standard cost (1)
            _ => {
                if base_cost > 0 {
                    base_cost * 2
                } else {
                    0
                }
            }
        }
    }
}
