use crate::logic::cards::behavior::CardBehavior;
use crate::types::GameState;

pub struct C16StrongHeadwind;

impl CardBehavior for C16StrongHeadwind {
    fn get_hit_threshold(&self, _state: &GameState) -> u32 {
        5
    }
}
