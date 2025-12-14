use super::ScoreDetails;
use sint_core::logic::cards::get_behavior;
use sint_core::types::CardSentiment;
use sint_core::types::{GamePhase, GameState, HazardType, MAX_HULL};

#[derive(Debug, Clone)]
pub struct RlScoringWeights {
    pub victory_reward: f64,
    pub defeat_penalty: f64,
    pub boss_damage_reward: f64,
    pub hull_damage_penalty: f64,
    pub fire_extinguish_reward: f64,
    pub system_repair_reward: f64,
    pub situation_resolve_reward: f64,
    pub item_pickup_reward: f64,
    pub item_drop_penalty: f64,
    pub shooting_reward: f64,
    pub turn_penalty: f64,
    pub step_penalty: f64,
}

impl Default for RlScoringWeights {
    fn default() -> Self {
        Self {
            victory_reward: 100.0,
            defeat_penalty: -100.0,
            boss_damage_reward: 10.0,
            hull_damage_penalty: 5.0,
            fire_extinguish_reward: 1.0,
            system_repair_reward: 1.0,
            situation_resolve_reward: 5.0,
            item_pickup_reward: 0.1,
            item_drop_penalty: 0.15,
            shooting_reward: 0.2,
            turn_penalty: 0.5,
            step_penalty: 0.01,
        }
    }
}

/// RL-Specific Scoring
/// Designed to be dense, normalized, and provide clear signals for reinforcement learning.
pub fn score_rl(
    parent: &GameState,
    current: &GameState,
    history: &[&(sint_core::types::PlayerId, sint_core::types::GameAction)],
    weights: &RlScoringWeights,
) -> ScoreDetails {
    let mut details = ScoreDetails::default();

    // 1. Terminal States (Immediate large signals)
    if current.phase == GamePhase::Victory {
        details.vitals = weights.victory_reward;
        details.total = details.vitals;
        return details;
    }
    if current.phase == GamePhase::GameOver || current.hull_integrity <= 0 {
        details.vitals = weights.defeat_penalty;
        details.total = details.vitals;
        return details;
    }

    // 2. Boss Damage Delta
    let damage_dealt = (parent.enemy.hp - current.enemy.hp) as f64;
    if damage_dealt > 0.0 {
        details.offense += damage_dealt * weights.boss_damage_reward;
    }

    // 3. Action-based Rewards (Dense signals from history)
    if let Some((_pid, action)) = history.last() {
        match action {
            sint_core::types::GameAction::Shoot => {
                details.offense += weights.shooting_reward;
            }
            _ => {}
        }
    }

    // 3. Hull Integrity Delta
    let hull_lost = (parent.hull_integrity - current.hull_integrity) as f64;
    if hull_lost > 0.0 {
        details.vitals -= hull_lost * weights.hull_damage_penalty;
    }

    // 4. Hazards Delta
    let parent_fire: usize = parent
        .map
        .rooms
        .values()
        .map(|r| r.hazards.iter().filter(|h| **h == HazardType::Fire).count())
        .sum();
    let current_fire: usize = current
        .map
        .rooms
        .values()
        .map(|r| r.hazards.iter().filter(|h| **h == HazardType::Fire).count())
        .sum();

    if current_fire < parent_fire {
        details.hazards += (parent_fire - current_fire) as f64 * weights.fire_extinguish_reward;
    }

    let parent_broken = parent.map.rooms.values().filter(|r| r.is_broken).count();
    let current_broken = current.map.rooms.values().filter(|r| r.is_broken).count();

    if current_broken < parent_broken {
        details.hazards += (parent_broken - current_broken) as f64 * weights.system_repair_reward;
    }

    // 5. Situations Delta
    let parent_neg = parent
        .active_situations
        .iter()
        .filter(|c| get_behavior(c.id).get_sentiment() == CardSentiment::Negative)
        .count();
    let current_neg = current
        .active_situations
        .iter()
        .filter(|c| get_behavior(c.id).get_sentiment() == CardSentiment::Negative)
        .count();

    if current_neg < parent_neg {
        details.situations += (parent_neg - current_neg) as f64 * weights.situation_resolve_reward;
    }

    // 6. Logistics (Incremental progress)
    let parent_items: usize = parent.players.values().map(|p| p.inventory.len()).sum();
    let current_items: usize = current.players.values().map(|p| p.inventory.len()).sum();
    if current_items > parent_items {
        details.logistics += (current_items - parent_items) as f64 * weights.item_pickup_reward;
    } else if current_items < parent_items {
        details.logistics -= (parent_items - current_items) as f64 * weights.item_drop_penalty;
    }

    // 7. Time/Efficiency penalties
    if current.turn_count > parent.turn_count {
        details.progression -= weights.turn_penalty;
    }
    details.progression -= weights.step_penalty;

    // Additional "Survival" density: reward remaining alive each turn
    details.vitals += (current.hull_integrity as f64 / MAX_HULL as f64) * 0.1;

    details.total = details.vitals
        + details.hazards
        + details.offense
        + details.logistics
        + details.situations
        + details.progression;

    details
}
