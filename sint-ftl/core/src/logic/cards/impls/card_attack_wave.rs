use crate::{
    logic::cards::behavior::CardBehavior,
    types::{Card, CardId, CardSolution, CardType, GameState},
};

pub struct AttackWaveCard;

impl CardBehavior for AttackWaveCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::AttackWave,
            title: "Attack Wave".to_owned(),
            description: "Enemy attacks twice this round!".to_owned(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                target_system: None, // Any room
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
