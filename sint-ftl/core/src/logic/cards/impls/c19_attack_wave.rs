use crate::logic::cards::behavior::CardBehavior;
use crate::types::GameState;

pub struct C19AttackWave;

use crate::types::{Card, CardId, CardSolution, CardType};

impl CardBehavior for C19AttackWave {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::AttackWave,
            title: "Attack Wave".to_string(),
            description: "Enemy attacks twice this round!".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(8),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn get_enemy_attack_count(&self, _state: &GameState) -> u32 {
        2
    }
}
