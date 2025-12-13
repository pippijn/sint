use sint_core::types::{GamePhase, GameState, HazardType, ItemType};

/// RHEA-Specific Scoring
/// Focuses on:
/// 1. Victory (Huge Reward)
/// 2. Boss Damage (High Reward)
/// 3. Hull Integrity (High Penalty for loss, medium reward for keeping)
/// 4. Action Efficiency (Penalty for wasting time)
/// 5. Hazard Control (Penalty for active hazards)
///
/// Unlike Beam search, we don't micro-manage "distance to room" or "holding item" here as much.
/// We rely on the evolution to find the sequence that leads to DAMAGE.
pub fn score_rhea(state: &GameState) -> f64 {
    // 1. Terminal States
    if state.phase == GamePhase::Victory {
        return 1_000_000.0 + (state.hull_integrity as f64 * 1000.0);
    }
    if state.phase == GamePhase::GameOver || state.hull_integrity <= 0 {
        return -1_000_000.0;
    }

    let mut score = 0.0;

    // 2. Boss Damage (Primary Goal)
    let damage_dealt = (state.enemy.max_hp - state.enemy.hp) as f64;
    score += damage_dealt * 5000.0; // Big points for blood

    // 3. Hull Integrity (Survival)
    // Non-linear penalty for low hull
    if state.hull_integrity < 10 {
        score -= (10.0 - state.hull_integrity as f64).powf(2.0) * 1000.0;
    } else {
        score += state.hull_integrity as f64 * 100.0;
    }

    // 4. Hazards (Fire is bad)
    let fire_count: usize = state
        .map
        .rooms
        .values()
        .map(|r| r.hazards.iter().filter(|h| **h == HazardType::Fire).count())
        .sum();
    score -= fire_count as f64 * 2000.0;

    // 5. Ammo Logistics (Heuristic hint)
    // RHEA struggles to find "Pick Up" without immediate reward.
    // We give a small hint for holding ammo.
    let total_ammo: usize = state
        .players
        .values()
        .map(|p| {
            p.inventory
                .iter()
                .filter(|i| **i == ItemType::Peppernut)
                .count()
        })
        .sum();
    score += total_ammo as f64 * 200.0;

    // 6. Time Penalty (Prevent stalling)
    score -= state.turn_count as f64 * 100.0;

    score
}
