use crate::logic::cards::behavior::CardBehavior;
use crate::types::{Card, CardId, CardSolution, CardType, GameState, ItemType};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub struct TantrumCard;

impl CardBehavior for TantrumCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Tantrum,
            title: "Tantrum".to_string(),
            description: "Max Chaos. Random events every turn.".to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: None,
                ap_cost: 1,
                item_cost: Some(ItemType::Peppernut),
                required_players: 1,
            }),
        }
    }

    // Effect: Max Chaos. The Toddler throws her die EVERY TURN.
    // We'll interpret this as: Enemy targets a RANDOM room every turn, ignoring the Telegraph.
    // We hook into `resolve_enemy_attack` by modifying the target right before execution?
    // Or we modify `get_enemy_attack_count`?
    // Let's implement `on_round_end` to randomize the NEXT attack (Telegraph).
    // `advance_phase` generates telegraph in `EnemyTelegraph`.
    // We can override it?
    // Let's use `check_resolution` to redirect the attack?
    // But `check_resolution` is for Player actions.

    // We'll use a hack: In `on_activate` and `on_round_start`, we scramble the current `next_attack`.

    fn on_activate(&self, state: &mut GameState) {
        self.scramble_attack(state);
    }

    fn on_round_start(&self, state: &mut GameState) {
        self.scramble_attack(state);
    }
}

impl TantrumCard {
    fn scramble_attack(&self, state: &mut GameState) {
        if let Some(attack) = &mut state.enemy.next_attack {
            let mut rng = StdRng::seed_from_u64(state.rng_seed);
            attack.target_room =
                rng.gen_range(crate::logic::MIN_ROOM_ID..=crate::logic::MAX_ROOM_ID);
            state.rng_seed = rng.gen();
        }
    }
}
