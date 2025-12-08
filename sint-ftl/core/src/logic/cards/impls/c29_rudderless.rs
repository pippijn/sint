use crate::logic::cards::behavior::CardBehavior;

pub struct C29Rudderless;

impl CardBehavior for C29Rudderless {
    // Effect: Hard Hits. 1 extra token per damage.
    // Hook into `resolve_enemy_attack`?
    // We haven't hooked `resolve_enemy_attack` damage calculation to registry yet.
    // `resolve_enemy_attack` currently hardcodes 1 Fire/Water.
    // We should add a `modify_damage` hook or similar?
    // Or `get_enemy_attack_effect_modifier`?
    // Let's stick to base for now.
}
