use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Action, GameState};
use crate::GameError;

pub struct C01AfternoonNap;

impl CardBehavior for C01AfternoonNap {
    fn validate_action(&self, state: &GameState, player_id: &str, action: &Action) -> Result<(), GameError> {
        // Logic: "The Reader" cannot spend AP.
        // Definition of "Reader": The player whose ID is lexicographically first.
        let mut sorted_ids: Vec<&String> = state.players.keys().collect();
        sorted_ids.sort();
        
        let reader_id = sorted_ids.first();
        
        if let Some(&rid) = reader_id {
            if rid == player_id {
                // Check if action costs AP
                // Hardcoded knowledge of costs here, or we'd need to invoke cost calc?
                // But validate is called BEFORE cost calc in apply_action.
                // We'll duplicate the base cost logic slightly or just block all non-free actions.
                let is_free = matches!(action, 
                    Action::Chat { .. } | 
                    Action::VoteReady { .. } | 
                    Action::Pass | 
                    Action::Join { .. } | 
                    Action::SetName { .. } |
                    Action::FullSync { .. } |
                    Action::Undo { .. }
                );
                
                if !is_free {
                    return Err(GameError::InvalidAction(
                        "The Reader (You) is asleep and cannot spend AP!".to_string()
                    ));
                }
            }
        }
        Ok(())
    }
}
