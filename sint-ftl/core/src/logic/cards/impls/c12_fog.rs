use crate::logic::cards::behavior::CardBehavior;

pub struct C12FogBank;

impl CardBehavior for C12FogBank {
    // Effect: Cannot see Enemy Intent (Telegraph disabled).
    // This is purely visual/API side masking, or handled in `EnemyTelegraph` phase logic.
    // If the game engine generates a telegraph, this card hides it?
    // Or prevents it from being generated?
    // "Telegraph disabled".
    // We can implement `on_activate` to clear `next_attack`?
    // But `next_attack` is generated *during* MorningReport/Telegraph phase.
    // If this is a Situation, it persists.
    // We'll leave it as a marker for now, as `CardBehavior` doesn't strictly hook into `advance_phase` telegraph generation logic yet.
    // But we can update `advance_phase` to check for this card.
}
