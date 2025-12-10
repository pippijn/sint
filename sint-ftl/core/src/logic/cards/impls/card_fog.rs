use crate::{
    logic::cards::behavior::CardBehavior,
    types::{AttackEffect, Card, CardId, CardSolution, CardType, EnemyAttack, GameState},
};
use rand::{rngs::StdRng, Rng, SeedableRng};

pub struct FogBankCard;

impl CardBehavior for FogBankCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::FogBank,
            title: "Fog Bank".to_string(),
            description: "Cannot see Enemy Intent (Telegraph disabled).".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::types::SystemType::Bow.as_u32()),
                ap_cost: 2,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn modify_telegraph(&self, attack: &mut EnemyAttack) {
        // Mask the attack
        attack.target_room = 0; // Unknown
        attack.effect = AttackEffect::Hidden;
    }

    fn resolve_telegraph(&self, state: &mut GameState, attack: &mut EnemyAttack) {
        if let AttackEffect::Hidden = &attack.effect {
            // Reveal/Generate the attack now
            let mut rng = StdRng::seed_from_u64(state.rng_seed);
            // 2d6 distribution (2-12)
            let target_room = rng.gen_range(1..=6) + rng.gen_range(1..=6);
            state.rng_seed = rng.gen();

            attack.target_room = target_room;
            attack.effect = AttackEffect::Fireball;
        }
    }
}
