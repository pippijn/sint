use crate::logic::cards::behavior::CardBehavior;
use crate::types::GameState;

pub struct C19AttackWave;

impl CardBehavior for C19AttackWave {
    fn get_enemy_attack_count(&self, _state: &GameState) -> u32 {
        2
    }
}
