use sint_core::types::{GamePhase, GameState, HazardType, ItemType};

#[derive(Debug, Clone)]
pub struct RheaScoringWeights {
    pub victory_base: f64,
    pub victory_hull_mult: f64,
    pub defeat_penalty: f64,

    pub boss_damage_reward: f64,

    pub hull_critical_threshold: i32,
    pub hull_critical_penalty_base: f64,
    pub hull_normal_reward: f64,

    pub fire_penalty: f64,
    pub ammo_holding_reward: f64,
    pub turn_penalty: f64,
}

impl Default for RheaScoringWeights {
    fn default() -> Self {
        Self {
            victory_base: 1_000_000.0,
            victory_hull_mult: 1000.0,
            defeat_penalty: -1_000_000.0,

            boss_damage_reward: 5000.0,

            hull_critical_threshold: 10,
            hull_critical_penalty_base: 1000.0,
            hull_normal_reward: 100.0,

            fire_penalty: 2000.0,
            ammo_holding_reward: 200.0,
            turn_penalty: 100.0,
        }
    }
}

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
pub fn score_rhea(state: &GameState, weights: &RheaScoringWeights) -> f64 {
    // 1. Terminal States
    if state.phase == GamePhase::Victory {
        return weights.victory_base + (state.hull_integrity as f64 * weights.victory_hull_mult);
    }
    if state.phase == GamePhase::GameOver || state.hull_integrity <= 0 {
        return weights.defeat_penalty;
    }

    let mut score = 0.0;

    // 2. Boss Damage (Primary Goal)
    let damage_dealt = (state.enemy.max_hp - state.enemy.hp) as f64;
    score += damage_dealt * weights.boss_damage_reward; // Big points for blood

    // 3. Hull Integrity (Survival)
    // Non-linear penalty for low hull
    if state.hull_integrity < weights.hull_critical_threshold {
        score -= (weights.hull_critical_threshold as f64 - state.hull_integrity as f64).powf(2.0)
            * weights.hull_critical_penalty_base;
    } else {
        score += state.hull_integrity as f64 * weights.hull_normal_reward;
    }

    // 4. Hazards (Fire is bad)
    let fire_count: usize = state
        .map
        .rooms
        .values()
        .map(|r| r.hazards.iter().filter(|h| **h == HazardType::Fire).count())
        .sum();
    score -= fire_count as f64 * weights.fire_penalty;

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
    score += total_ammo as f64 * weights.ammo_holding_reward;

    // 6. Time Penalty (Prevent stalling)
    score -= state.turn_count as f64 * weights.turn_penalty;

    score
}
