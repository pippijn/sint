use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{
        AttackEffect, Card, CardId, CardSolution, CardType, EnemyAttack, GameState, SystemType,
    },
};
use rand::{Rng, SeedableRng, rngs::StdRng};

pub struct FogBankCard;

impl CardBehavior for FogBankCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::FogBank,
            title: "Fog Bank".to_owned(),
            description: "Cannot see Enemy Intent (Telegraph disabled).".to_owned(),
            card_type: CardType::Situation,
            options: vec![].into(),
            solution: Some(CardSolution {
                target_system: Some(SystemType::Bow),
                ap_cost: 2,
                item_cost: None,
                required_players: 1,
            }),
            affected_player: None,
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
            let roll = rng.random_range(1..=6) + rng.random_range(1..=6);
            state.rng_seed = rng.random();

            if let Some(sys) = SystemType::from_u32(roll) {
                attack.target_system = Some(sys);
                attack.effect = AttackEffect::Fireball;
                attack.target_room = find_room_with_system(state, sys).unwrap_or(0);
            } else {
                attack.target_system = None;
                attack.effect = AttackEffect::Miss;
                attack.target_room = 0;
            }
        }
    }
}
