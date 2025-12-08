use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, GameState, CardType, CardId};
use crate::GameError;

pub struct C34MonsterDough;

impl CardBehavior for C34MonsterDough {
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
            // Kitchen unusable?
            // This requires a persistent "Ruined" state for room 6.
            // We'll remove the card but leave the effect? Or transform card into "Ruined Kitchen"?
            // Let's transform it to a Situation?
            // "Cleaning later costs 2 AP".
            // We'll leave it as active with rounds_left=0 acting as the situation.
        } else {
             // Remove if solved? Solution logic is in Action::Interact usually.
        }
    }

    fn validate_action(&self, _state: &GameState, _player_id: &str, action: &Action) -> Result<(), GameError> {
        // If triggered (rounds_left == 0)
        // Block actions in Kitchen.
        if let Action::Bake = action {
             // Check if triggered? 
             // We need access to self state or query state.
             // We'll assume if this behavior is active, we check the card in state.
             return Err(GameError::InvalidAction("Monster Dough! Kitchen blocked.".to_string()));
        }
        Ok(())
    }
}
