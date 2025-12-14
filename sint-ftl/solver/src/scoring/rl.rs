use super::ScoreDetails;
use sint_core::logic::actions::action_cost;
use sint_core::logic::cards::get_behavior;
use sint_core::types::CardSentiment;
use sint_core::types::{GamePhase, GameState, HazardType};

#[derive(Debug, Clone)]
pub struct RlScoringWeights {
    pub victory_reward: f64,
    pub defeat_penalty: f64,
    pub boss_damage_reward: f64,
    pub hull_damage_penalty: f64,
    pub hazard_cleanup_reward: f64,
    pub system_repair_reward: f64,
    pub system_health_restore_reward: f64,
    pub situation_resolve_reward: f64,
    pub item_pickup_reward: f64,
    pub item_drop_penalty: f64,
    pub defensive_action_reward: f64,
    pub constant_survival_reward: f64,
    pub turn_penalty: f64,
    pub step_penalty_per_ap: f64,
}

impl Default for RlScoringWeights {
    fn default() -> Self {
        Self {
            // Ultimate goal, large enough to dominate any single trajectory.
            victory_reward: 1000.0,
            // High penalty to discourage hull loss/crew wipe.
            defeat_penalty: -200.0,
            // Dense signal for offensive progress; scales with enemy HP.
            boss_damage_reward: 5.0,
            // High cost per hull point to encourage defense and repairs.
            hull_damage_penalty: 10.0,
            // Reward for extinguishing/moping; outweighs the AP cost.
            hazard_cleanup_reward: 1.5,
            // Reward for restoring a broken system to a functional state.
            system_repair_reward: 1.5,
            // Incremental reward for partial system repairs (system health increase).
            system_health_restore_reward: 0.5,
            // Reward for solving negative card events (Situations).
            situation_resolve_reward: 5.0,
            // Net positive (+0.3) for gathering resources after 1 AP step penalty.
            item_pickup_reward: 0.4,
            // Anti-hacking: must be > pickup + survival (0.4 + 0.1) to make loops negative.
            item_drop_penalty: 0.6,
            // Immediate feedback for using shields/evasion against telegraphed threats.
            defensive_action_reward: 0.5,
            // Density signal for staying alive; avoids bias towards high-health states.
            constant_survival_reward: 0.1,
            // Discourages stalling; must be > survival bonus + turn impact.
            turn_penalty: 2.2,
            // Scales the time-cost of actions linearly with their AP usage.
            step_penalty_per_ap: 0.1,
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
    let mut last_ap_cost = 1;
    if let Some((pid, action)) = history.last() {
        last_ap_cost = action_cost(parent, pid, action);

        match action {
            sint_core::types::GameAction::RaiseShields
            | sint_core::types::GameAction::EvasiveManeuvers => {
                // Reward defensive actions if an attack is telegraphed
                if current
                    .enemy
                    .next_attack
                    .as_ref()
                    .is_some_and(|a| a.target_room.is_some())
                {
                    details.vitals += weights.defensive_action_reward;
                }
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
        details.hazards += (parent_fire - current_fire) as f64 * weights.hazard_cleanup_reward;
    }

    let parent_water: usize = parent
        .map
        .rooms
        .values()
        .map(|r| {
            r.hazards
                .iter()
                .filter(|h| **h == HazardType::Water)
                .count()
        })
        .sum();
    let current_water: usize = current
        .map
        .rooms
        .values()
        .map(|r| {
            r.hazards
                .iter()
                .filter(|h| **h == HazardType::Water)
                .count()
        })
        .sum();

    if current_water < parent_water {
        details.hazards += (parent_water - current_water) as f64 * weights.hazard_cleanup_reward;
    }

    let parent_broken = parent.map.rooms.values().filter(|r| r.is_broken).count();
    let current_broken = current.map.rooms.values().filter(|r| r.is_broken).count();

    if current_broken < parent_broken {
        details.hazards += (parent_broken - current_broken) as f64 * weights.system_repair_reward;
    }

    let parent_health: u32 = parent.map.rooms.values().map(|r| r.system_health).sum();
    let current_health: u32 = current.map.rooms.values().map(|r| r.system_health).sum();

    if current_health > parent_health {
        details.hazards +=
            (current_health - parent_health) as f64 * weights.system_health_restore_reward;
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
    // Scale step penalty by the AP cost of the action
    details.progression -= (last_ap_cost as f64) * weights.step_penalty_per_ap;

    // Additional "Survival" density: constant reward for remaining alive
    details.vitals += weights.constant_survival_reward;

    details.total = details.vitals
        + details.hazards
        + details.offense
        + details.logistics
        + details.situations
        + details.progression;

    details
}
