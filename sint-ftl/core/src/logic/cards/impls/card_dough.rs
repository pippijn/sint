use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameAction, GameState, SystemType},
    GameError,
};

pub struct MonsterDoughCard;

impl CardBehavior for MonsterDoughCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::MonsterDough,
            title: "Monster Dough".to_owned(),
            description: "Boom: Kitchen is unusable.".to_owned(),
            card_type: CardType::Timebomb { rounds_left: 3 },
            options: vec![],
            solution: Some(CardSolution {
                target_system: Some(SystemType::Kitchen),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
            affected_player: None,
        }
    }

    fn validate_action(
        &self,
        state: &GameState,
        _player_id: &str,
        action: &GameAction,
    ) -> Result<(), GameError> {
        // If triggered (rounds_left == 0)
        // Block actions in Kitchen.
        if let GameAction::Bake = action {
            let triggered = state.active_situations.iter().any(|c| {
                c.id == CardId::MonsterDough
                    && matches!(c.card_type, CardType::Timebomb { rounds_left: 0 })
            });
            if triggered {
                return Err(GameError::InvalidAction(
                    "Monster Dough! Kitchen blocked.".to_owned(),
                ));
            }
        }
        Ok(())
    }

    fn on_round_end(&self, state: &mut GameState) {
        let mut triggered = false;
        for card in state.active_situations.iter_mut() {
            if card.id == CardId::MonsterDough {
                if let CardType::Timebomb { rounds_left } = &mut card.card_type {
                    if *rounds_left > 0 {
                        *rounds_left -= 1;
                        if *rounds_left == 0 {
                            triggered = true;
                        }
                    }
                }
            }
        }

        if triggered {
            // Kitchen is unusable.
            // Note: We keep rounds_left at 0 to signal the persistent blocked state to validate_action.
        } else {
            // Remove if solved? Solution logic is in Action::Interact usually.
        }
    }
}
