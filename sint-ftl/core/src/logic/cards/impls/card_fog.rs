use crate::{
    logic::{cards::behavior::CardBehavior, find_room_with_system},
    types::{
        AttackEffect, Card, CardId, CardSolution, CardType, EnemyAttack, GameState, SystemType,
    },
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
                target_system: Some(SystemType::Bow),
                ap_cost: 2,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn validate_action(
        &self,
        state: &GameState,
        player_id: &str,
        action: &crate::types::GameAction,
    ) -> Result<(), crate::GameError> {
        if let crate::types::GameAction::Interact = action {
            let p = state.players.get(player_id).unwrap();
            let bow = find_room_with_system(state, SystemType::Bow);
            if Some(p.room_id) != bow {
                return Err(crate::GameError::InvalidAction(
                    "Must be in The Bow to clear Fog.".to_string(),
                ));
            }
        }
        Ok(())
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
            let roll = rng.gen_range(1..=6) + rng.gen_range(1..=6);
            state.rng_seed = rng.gen();

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
