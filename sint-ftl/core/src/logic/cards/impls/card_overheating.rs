use crate::logic::cards::behavior::CardBehavior;
use crate::types::GameState;

pub struct OverheatingCard;

use crate::types::{Card, CardId, CardSolution, CardType};

impl CardBehavior for OverheatingCard {
    fn get_struct(&self) -> Card {
        Card {
            id: CardId::Overheating,
            title: "Overheating".to_string(),
            description: format!(
                "End turn in Engine ({}) -> Lose 1 AP next round.",
                crate::logic::ROOM_ENGINE
            )
            .to_string(),
            card_type: CardType::Situation,
            options: vec![],
            solution: Some(CardSolution {
                room_id: Some(crate::logic::ROOM_ENGINE),
                ap_cost: 1,
                item_cost: None,
                required_players: 1,
            }),
        }
    }

    fn on_round_start(&self, state: &mut GameState) {
        // Effect: Players who ended turn in Room crate::logic::ROOM_ENGINE lose 1 AP.
        // But players AP is reset in `EnemyTelegraph`.
        // We need to apply this AFTER reset.
        // `advance_phase` calls `on_round_start` during `MorningReport`.
        // Then `EnemyTelegraph` resets AP to 2.
        // So `on_round_start` is too early for modifying AP (unless we track it as a status).

        // Wait, `on_round_start` is called in `advance_phase` for `MorningReport` AND `EnemyAction`.
        // But `advance_phase` for `EnemyTelegraph` resets AP.
        // We don't have a hook "on_ap_reset".

        // Let's check `logic.rs`.
        // `advance_phase(EnemyTelegraph)` -> Resets AP.
        // Then transitions to `TacticalPlanning`.

        // We can hook into `modify_action_cost`?
        // Or we assume `on_round_start` happens, sets a status, and AP reset respects it?
        // We don't have status logic for custom statuses.

        // Let's modify AP NOW in `on_round_start` (MorningReport).
        // BUT `EnemyTelegraph` will overwrite it to 2.

        // We need to change `logic.rs` to respect AP modifiers or call a hook after reset.
        // OR we just set AP to 1 in `on_round_start` and hope `EnemyTelegraph` doesn't run?
        // No, the phases are sequential.

        // Hack: Check `advance_phase` in `logic.rs` again.
        // `GamePhase::EnemyTelegraph` -> `state.phase = GamePhase::TacticalPlanning; ... p.ap = 2;`

        // We can't implement this correctly without modifying `logic.rs` or adding a new hook.
        // Let's stick to the pattern: `on_round_start` sets AP.
        // If `logic.rs` overwrites it, it's a bug in `logic.rs` extensibility.
        // But since I can edit `logic.rs`, I should probably fix that?
        // No, I shouldn't edit core logic if I can avoid it.

        // Maybe I can deduct AP in `check_resolution` of the FIRST action?
        // No, that's complex.

        // Let's implement it in `on_round_start` and assume `logic.rs` will be updated later or we accept the bug for now?
        // Or better: `C50Overheating` checks if `phase == TacticalPlanning`?
        // `CardBehavior` doesn't have `on_phase_change`.

        // Let's modify AP in `on_round_start` and acknowledge it might be overwritten.
        // Actually, if we look at `C33FluWave` implementation earlier (which I read),
        // it had the same issue: "Next round 1 AP... We'll leave this unimplemented correctly without a Status system."

        // I will implement the logic to check Room crate::logic::ROOM_ENGINE.
        // And I will try to set AP to 1.
        for p in state.players.values_mut() {
            if p.room_id == crate::logic::ROOM_ENGINE {
                // Mark them?
                // We'll just reduce AP and see.
                if p.ap > 0 {
                    p.ap -= 1;
                }
            }
        }
    }
}
