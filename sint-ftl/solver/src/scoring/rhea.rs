use super::ScoreDetails;
use sint_core::types::{GamePhase, GameState, HazardType, ItemType};

#[derive(Debug, Clone)]
pub struct RheaScoringWeights {
    pub victory_base: f64,
    pub victory_hull_mult: f64,
    pub defeat_penalty: f64,

    pub boss_damage_reward: f64,
    pub boss_level_reward: f64,

    pub hull_critical_threshold: i32,
    pub hull_critical_penalty_base: f64,
    pub hull_normal_reward: f64,

    pub fire_penalty: f64,
    pub water_penalty: f64,
    pub fainted_penalty: f64,
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
            boss_level_reward: 100_000.0,

            hull_critical_threshold: 10,
            hull_critical_penalty_base: 1000.0,
            hull_normal_reward: 100.0,

            fire_penalty: 2000.0,
            water_penalty: 500.0,
            fainted_penalty: 5000.0,
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
pub fn score_rhea(state: &GameState, weights: &RheaScoringWeights) -> ScoreDetails {
    let mut details = ScoreDetails::default();

    // 1. Terminal States
    if state.phase == GamePhase::Victory {
        details.vitals =
            weights.victory_base + (state.hull_integrity as f64 * weights.victory_hull_mult);
        details.total = details.vitals;
        return details;
    }
    if state.phase == GamePhase::GameOver || state.hull_integrity <= 0 {
        details.vitals = weights.defeat_penalty;
        details.total = details.vitals;
        return details;
    }

    // 2. Boss Damage (Primary Goal)
    let damage_dealt = (state.enemy.max_hp - state.enemy.hp) as f64;
    details.offense += damage_dealt * weights.boss_damage_reward;
    details.progression += state.boss_level as f64 * weights.boss_level_reward;

    // 3. Hull Integrity (Survival)
    // Non-linear penalty for low hull
    if state.hull_integrity < weights.hull_critical_threshold {
        details.panic -= (weights.hull_critical_threshold as f64 - state.hull_integrity as f64)
            .powf(2.0)
            * weights.hull_critical_penalty_base;
    } else {
        details.vitals += state.hull_integrity as f64 * weights.hull_normal_reward;
    }

    // 4. Hazards (Fire is bad for Hull, Water is bad for Items)
    let mut fire_count = 0;
    let mut water_count = 0;
    for room in state.map.rooms.values() {
        for hazard in &room.hazards {
            match hazard {
                HazardType::Fire => fire_count += 1,
                HazardType::Water => water_count += 1,
            }
        }
    }
    details.hazards -= fire_count as f64 * weights.fire_penalty;
    details.hazards -= water_count as f64 * weights.water_penalty;

    // 5. Player Status
    let fainted_count = state
        .players
        .values()
        .filter(|p| p.status.contains(&sint_core::types::PlayerStatus::Fainted))
        .count();
    details.vitals -= fainted_count as f64 * weights.fainted_penalty;

    // 6. Ammo Logistics (Heuristic hint)
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
    details.logistics += total_ammo as f64 * weights.ammo_holding_reward;

    // 7. Time Penalty (Prevent stalling)
    details.progression -= state.turn_count as f64 * weights.turn_penalty;

    details.total = details.vitals
        + details.hazards
        + details.offense
        + details.panic
        + details.logistics
        + details.progression;

    details
}
