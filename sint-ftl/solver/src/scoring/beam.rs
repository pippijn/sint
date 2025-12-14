use super::ScoreDetails;
use sint_core::logic::pathfinding::MapDistances;
use sint_core::logic::{cards::get_behavior, find_room_with_system};
use sint_core::types::{
    CardId, CardSentiment, GameAction, GamePhase, GameState, HazardType, ItemType, MAX_HULL,
    PlayerId, RoomId, SystemType,
};
use std::collections::{HashMap, HashSet};

/// Hyperparameters for the scoring function.
#[derive(Debug, Clone)]
pub struct BeamScoringWeights {
    // Vital Stats
    pub hull_integrity: f64,
    pub hull_delta_penalty: f64,
    pub enemy_hp: f64,
    pub player_hp: f64,
    pub ap_balance: f64,

    // Hazards
    pub fire_penalty_base: f64,
    pub fire_token_penalty: f64,
    pub water_penalty: f64,

    // Game State
    pub active_situation_penalty: f64,
    pub threat_player_penalty: f64,
    pub threat_system_penalty: f64,
    pub death_penalty: f64,

    // Roles & Positioning
    pub station_keeping_reward: f64, // New: Reward for being in assigned room
    pub gunner_base_reward: f64,
    pub gunner_per_ammo: f64,
    pub gunner_working_bonus: f64,
    pub gunner_distance_factor: f64,

    pub firefighter_base_reward: f64,
    pub firefighter_distance_factor: f64,

    pub healing_reward: f64,
    pub sickbay_distance_factor: f64,

    pub sentinel_reward: f64, // New: Reward for ending turn in critical room

    // Anti-Oscillation
    pub backtracking_penalty: f64,
    pub commitment_bonus: f64, // New: Reward for continuing a task

    // Situation Solving
    pub solution_solver_reward: f64,
    pub solution_distance_factor: f64,
    pub situation_logistics_reward: f64, // New: Reward for getting items needed for situations
    pub situation_resolved_reward: f64,  // New: Reward for the act of solving
    pub system_importance_multiplier: f64, // New: Multiplier for critical systems
    pub boss_killing_blow_reward: f64,   // New: Massive reward for finishing the boss
    pub inaction_penalty: f64,           // New: Penalty for repeated Pass/VoteReady

    // Logistics
    pub ammo_stockpile_reward: f64,
    pub loose_ammo_reward: f64,
    pub hazard_proximity_reward: f64,
    pub situation_exposure_penalty: f64,
    pub system_disabled_penalty: f64,
    pub shooting_reward: f64,

    // Exponents (Non-Linearity)
    pub hull_exponent: f64,
    pub fire_exponent: f64,
    pub cargo_repair_exponent: f64,
    pub hull_risk_exponent: f64,
    pub panic_fire_exponent: f64,           // New
    pub panic_hull_exponent: f64,           // New
    pub checkmate_exponent: f64,            // New
    pub hull_penalty_scaling: f64,          // New
    pub projected_hull_panic_exponent: f64, // New

    pub fire_panic_threshold_base: f64,          // New
    pub fire_panic_threshold_hull_scaling: f64, // New
    pub survival_only_multiplier: f64,          // New

    pub scavenger_reward: f64,
    pub repair_proximity_reward: f64,
    pub cargo_repair_incentive: f64,
    pub cargo_repair_proximity_reward: f64, // New: Reward for moving towards Cargo

    pub situation_exponent: f64, // New
    // Progression
    pub boss_level_reward: f64,
    pub turn_penalty: f64,
    pub step_penalty: f64,

    pub checkmate_system_bonus: f64, // New: Extra reward for operational systems when boss is low

    // Checkmate
    pub checkmate_threshold: f64,
    pub checkmate_multiplier: f64,
    pub checkmate_max_mult: f64, // New

    // Critical State / Panic
    pub critical_hull_threshold: f64,
    pub critical_hull_penalty_base: f64,
    pub critical_hull_penalty_per_hp: f64,
    pub critical_fire_threshold: usize,
    pub critical_fire_penalty_per_token: f64,
    pub critical_system_hazard_penalty: f64, // New: Penalty for hazard in critical system
    pub fire_in_critical_hull_penalty: f64,  // New: Extreme penalty for fire when hull is low
    pub critical_survival_mult: f64,
    pub critical_threat_mult: f64,

    pub item_juggling_penalty: f64, // New: Penalty for picking up and dropping the same item

    // Multipliers & Ranges
    pub fire_urgency_mult: f64,
    pub hazard_proximity_range: f64,
    pub gunner_dist_range: f64,
    pub gunner_per_ammo_mult: f64,
    pub gunner_en_route_mult: f64,
    pub gunner_wheelbarrow_penalty: f64,
    pub baker_wheelbarrow_mult: f64,

    pub threat_severe_reward: f64,
    pub threat_mitigated_reward: f64,
    pub threat_hull_risk_mult: f64,
    pub threat_shield_waste_penalty: f64,

    pub rest_round_hazard_multiplier: f64,
    pub rest_round_vitals_multiplier: f64,

    // New: Magic numbers from score_static
    pub victory_score: f64,
    pub game_over_score: f64,
    pub victory_hull_multiplier: f64,
    pub fire_spread_projected_damage: f64,
    pub max_hazard_multiplier: f64,
    pub critical_survival_boss_hp_threshold: f64,
    pub hull_risk_scaling_factor: f64,
    pub hull_risk_scaling_exponent: f64,
    pub hull_risk_max_mult: f64,
    pub bloodlust_hp_threshold: f64,
    pub bloodlust_multiplier: f64,
    pub checkmate_dampening_boss_hp_threshold: f64,
    pub panic_dampening_multiplier: f64,
    pub firefighter_panic_scaling: f64,

    pub bake_reward: f64,
    pub low_boss_hp_reward: f64,
    pub blocking_situation_multiplier: f64,
}

impl Default for BeamScoringWeights {
    fn default() -> Self {
        Self {
            hull_integrity: 1000000.0,
            hull_delta_penalty: 25000.0,
            enemy_hp: 100000.0,
            player_hp: 100.0,
            ap_balance: 100.0,

            // Hazards
            fire_penalty_base: 5000000.0,
            fire_token_penalty: 10000.0,
            water_penalty: 5000.0,

            // Situations & Threats
            active_situation_penalty: 10000.0,
            threat_player_penalty: 100000.0,
            threat_system_penalty: 500000.0,
            death_penalty: 5000.0,

            // Roles
            station_keeping_reward: 2000.0,
            gunner_base_reward: 5000.0,
            gunner_per_ammo: 10000.0,
            gunner_working_bonus: 5.0,
            gunner_distance_factor: 100.0,

            firefighter_base_reward: 500000.0,
            firefighter_distance_factor: 20.0,

            healing_reward: 5000.0,
            sickbay_distance_factor: 20.0,
            sentinel_reward: 5000.0,

            backtracking_penalty: 200.0,
            commitment_bonus: 50.0,

            solution_solver_reward: 200000.0,
            solution_distance_factor: 20.0,
            situation_logistics_reward: 10000.0,
            situation_resolved_reward: 500000.0,
            system_importance_multiplier: 5.0,
            boss_killing_blow_reward: 2000000000.0,
            inaction_penalty: 25000.0,

            ammo_stockpile_reward: 2000.0,
            loose_ammo_reward: 100.0,
            hazard_proximity_reward: 5.0,
            situation_exposure_penalty: 100.0,
            system_disabled_penalty: 1000000.0,
            shooting_reward: 10000.0,

            scavenger_reward: 5000.0,
            repair_proximity_reward: 2000.0,
            cargo_repair_incentive: 1000000.0,
            cargo_repair_proximity_reward: 5.0,
            item_juggling_penalty: 50000.0,
            situation_exponent: 2.0,

            boss_level_reward: 1000000.0,
            turn_penalty: 10.0,
            step_penalty: 1000.0,
            checkmate_system_bonus: 50000.0,

            checkmate_threshold: 3.0,
            checkmate_multiplier: 100.0,
            checkmate_max_mult: 2000.0,

            // Critical State
            critical_hull_threshold: 12.0,
            critical_hull_penalty_base: 20000.0,
            critical_hull_penalty_per_hp: 500000.0,
            critical_fire_threshold: 2,
            critical_fire_penalty_per_token: 2000.0,
            critical_system_hazard_penalty: 100000.0,
            fire_in_critical_hull_penalty: 10000000.0,
            critical_survival_mult: 0.4,
            critical_threat_mult: 5.0,

            // Exponents
            hull_exponent: 2.5,
            fire_exponent: 4.0,
            cargo_repair_exponent: 1.5,
            hull_risk_exponent: 1.1,
            panic_fire_exponent: 2.5,
            panic_hull_exponent: 2.0,
            checkmate_exponent: 1.8,
            hull_penalty_scaling: 1.1,
            projected_hull_panic_exponent: 4.0,

            fire_panic_threshold_base: 2.0,
            fire_panic_threshold_hull_scaling: 5.0,
            survival_only_multiplier: 0.4,

            // Multipliers
            fire_urgency_mult: 5.0,
            hazard_proximity_range: 20.0,
            gunner_dist_range: 20.0,
            gunner_per_ammo_mult: 0.2,
            gunner_en_route_mult: 0.1,
            gunner_wheelbarrow_penalty: 0.1,
            baker_wheelbarrow_mult: 2.0,

            threat_severe_reward: 2000000.0,
            threat_mitigated_reward: 10000.0,
            threat_hull_risk_mult: 0.5,
            threat_shield_waste_penalty: 100.0,

            rest_round_hazard_multiplier: 10.0,
            rest_round_vitals_multiplier: 10.0,

            victory_score: 2000000000.0,
            game_over_score: -1000000000.0,
            victory_hull_multiplier: 10000.0,
            fire_spread_projected_damage: 0.5,
            max_hazard_multiplier: 8.0,
            critical_survival_boss_hp_threshold: 10.0,
            hull_risk_scaling_factor: 10.0,
            hull_risk_scaling_exponent: 2.0,
            hull_risk_max_mult: 20.0,
            bloodlust_hp_threshold: 0.5,
            bloodlust_multiplier: 1.5,
            checkmate_dampening_boss_hp_threshold: 10.0,
            panic_dampening_multiplier: 5.0,
            firefighter_panic_scaling: 9.0,

            bake_reward: 5000.0,
            low_boss_hp_reward: 5000000.0,
            blocking_situation_multiplier: 50.0,
        }
    }
}
/// Calculates the total score for a search node, combining static state evaluation
/// and transition penalties/rewards (e.g., hull loss).
pub fn calculate_score(
    parent: &GameState,
    current: &GameState,
    history: &[&(PlayerId, GameAction)],
    weights: &BeamScoringWeights,
    distances: &MapDistances,
) -> ScoreDetails {
    let mut details = score_static(current, history, weights, distances);
    details += score_transition(parent, current, weights);

    // Commitment Bonus: Reward moving towards hazards
    if let Some((last_pid, last_act)) = history.last()
        && matches!(last_act, GameAction::Move { .. })
        && let (Some(old_p), Some(new_p)) =
            (parent.players.get(last_pid), current.players.get(last_pid))
    {
        let old_room = old_p.room_id;
        let new_room = new_p.room_id;

        // Identify targets (Fires)
        let mut target_rooms = HashSet::new();
        for r in current.map.rooms.values() {
            if r.hazards.contains(&HazardType::Fire) {
                target_rooms.insert(r.id);
            }
        }

        if !target_rooms.is_empty() {
            let old_dist = distances.min_distance(old_room, &target_rooms);
            let new_dist = distances.min_distance(new_room, &target_rooms);

            if new_dist < old_dist {
                details.logistics += weights.commitment_bonus;
                details.total += weights.commitment_bonus;
            }
        }
    }

    details
}

/// Calculates a score for a single state snapshot.
/// Higher is better.
pub fn score_static(
    state: &GameState,
    history: &[&(PlayerId, GameAction)],
    weights: &BeamScoringWeights,
    distances: &MapDistances,
) -> ScoreDetails {
    let mut details = ScoreDetails::default();

    // Terminal States
    // Terminal States: Increase magnitude to ensure separation from regular score swings.
    if state.phase == GamePhase::Victory {
        details.vitals = weights.victory_score + (state.hull_integrity as f64 * weights.victory_hull_multiplier);
        details.total = details.vitals;
        return details;
    }
    // SUICIDE PREVENTION: Death must be worse than any living hell.
    if state.phase == GamePhase::GameOver || state.hull_integrity <= 0 {
        details.total = weights.game_over_score;
        return details;
    }

    // PROJECTED DAMAGE: One hull damage per room on fire at end of round.
    // Enhanced: Account for potential fire spread if a room has 2+ fire tokens.
    let rooms_on_fire = state
        .map
        .rooms
        .values()
        .filter(|r| r.hazards.contains(&HazardType::Fire))
        .count();

    let fire_spread_sources = state
        .map
        .rooms
        .values()
        .filter(|r| r.hazards.iter().filter(|h| **h == HazardType::Fire).count() >= 2)
        .count();

    let fire_count_total: usize = state
        .map
        .rooms
        .values()
        .map(|r| r.hazards.iter().filter(|h| **h == HazardType::Fire).count())
        .sum();

    // PROJECTED DAMAGE: One hull damage per room on fire at end of round.
    // If Overheating is active, damage is per fire TOKEN.
    let is_overheating = state
        .active_situations
        .iter()
        .any(|c| c.id == CardId::Overheating);
    let fire_damage = if is_overheating {
        fire_count_total
    } else {
        rooms_on_fire
    };

    let projected_hull =
        state.hull_integrity - fire_damage as i32 - (fire_spread_sources as f64 * weights.fire_spread_projected_damage) as i32;
    // If fires will kill us this round, apply massive penalty but don't prune yet.
    // This allows "death or glory" plays to win before the fire damage applies.
    if projected_hull <= 0 && state.phase != GamePhase::Victory {
        details.panic += weights.game_over_score;
    }

    // Use PROJECTED hull for scaling
    let missing_hull_percent =
        ((MAX_HULL as f64 - projected_hull as f64) / MAX_HULL as f64).max(0.0);

    // Continuous Multipliers:
    // survival_multiplier: 1.0 at full health, down to critical_survival_mult at 0 health.
    let survival_multiplier = 1.0 - (missing_hull_percent * (1.0 - weights.critical_survival_mult));

    // hazard_multiplier: 1.0 at full health, up to 1.0 / critical_survival_mult at 0 health.
    // DAMPENED: Use a lower ceiling for the hazard multiplier to prevent -100M scores.
    let hazard_multiplier = 1.0 + (missing_hull_percent * (weights.max_hazard_multiplier - 1.0));

    // Penalty scaling based on hull integrity: lower hull = higher penalties for everything else.
    // Use PROJECTED hull.
    let hull_penalty_scaler =
        (MAX_HULL as f64 / (projected_hull as f64).max(1.0)).powf(weights.hull_penalty_scaling);

    // threat_mult: Scale threat penalties smoothly, but don't stack hull_penalty_scaler here yet.
    let threat_mult = 1.0 + (missing_hull_percent * (weights.critical_threat_mult - 1.0));

    // Use a threshold only for the 'panic' additive penalty
    let is_critical = (projected_hull as f64) <= weights.critical_hull_threshold;

    // --- Projected Hull Panic ---
    // Smooth exponential penalty as projected hull approaches zero.
    let projected_hull_panic = if projected_hull < MAX_HULL {
        let risk_factor =
            (MAX_HULL as f64 - projected_hull as f64) / (MAX_HULL as f64 - 1.0).max(1.0);
        risk_factor.powf(weights.projected_hull_panic_exponent) * weights.critical_hull_penalty_base
    } else {
        0.0
    };
    details.panic -= projected_hull_panic;

    // --- Checkmate Heuristic ---
    let mut checkmate_mult = 1.0;
    if (state.enemy.hp as f64) <= weights.checkmate_threshold && state.enemy.hp > 0 {
        let progress = weights.checkmate_threshold / (state.enemy.hp as f64);
        checkmate_mult = (progress.powf(weights.checkmate_exponent) * weights.checkmate_multiplier)
            .min(weights.checkmate_max_mult);

        // Survival Override: If we are in critical condition, dampen bloodlust unless the boss is almost dead.
        if is_critical {
            if (state.enemy.hp as f64) > weights.critical_survival_boss_hp_threshold {
                checkmate_mult = 1.0;
            } else {
                // Do NOT dampen if the boss is at 10 HP or less. Finishing the boss IS survival.
            }
        }
    }

    // --- Critical State Heuristic ---
    // Override everything if we are about to die.
    // Survival is absolute. Do not reduce panic based on enemy health.

    if is_critical {
        let mut panic_penalty = weights.critical_hull_penalty_base;
        let deficit = (weights.critical_hull_threshold + 1.0 - projected_hull as f64).max(0.0);
        panic_penalty += deficit.powf(weights.panic_hull_exponent)
            * weights.critical_hull_penalty_per_hp
            * hull_penalty_scaler;

        // New: If hull is critical AND there is any fire, add an extreme penalty.
        // This prevents the AI from ignoring "small" fires when it's one hit from death.
        if rooms_on_fire > 0 {
            panic_penalty += weights.fire_in_critical_hull_penalty * hull_penalty_scaler;
        }

        details.panic -= panic_penalty;
    }

    // Dynamic Hazard Threshold: Panic triggers earlier if hull is low.
    let dynamic_fire_threshold = (weights.critical_fire_threshold as f64
        * (projected_hull as f64 / MAX_HULL as f64))
        .max(1.0);

    if fire_count_total as f64 >= dynamic_fire_threshold {
        // Massive penalty for uncontrolled fire
        let excess_fire = (fire_count_total as f64 - dynamic_fire_threshold + 1.0).max(1.0);
        let fire_panic = excess_fire.powf(weights.panic_fire_exponent)
            * weights.critical_fire_penalty_per_token
            * hull_penalty_scaler;
        details.panic -= fire_panic;
    }

    // --- Fire Panic Mode ---
    let fire_panic_threshold = weights.fire_panic_threshold_base
        + (state.hull_integrity as f64 / MAX_HULL as f64) * weights.fire_panic_threshold_hull_scaling;
    let in_fire_panic = rooms_on_fire as f64 > fire_panic_threshold;

    // --- 1. Vital Stats & Progression ---
    // Non-linear Hull Penalty: Penalize missing hull exponentially (Square of missing health)
    // Use PROJECTED hull
    let missing_hull = (MAX_HULL as f64 - projected_hull as f64).max(0.0);
    let mut hull_penalty = missing_hull.powf(weights.hull_exponent) * weights.hull_integrity;
    if state.is_resting {
        hull_penalty *= weights.rest_round_vitals_multiplier;
    }
    details.vitals -= hull_penalty;

    details.progression += state.boss_level as f64 * weights.boss_level_reward;

    // Bloodlust: If enemy is low, increase reward for damage
    let enemy_hp_percent = if state.enemy.max_hp > 0 {
        state.enemy.hp as f64 / state.enemy.max_hp as f64
    } else {
        0.0
    };
    let bloodlust_mult = if enemy_hp_percent < weights.bloodlust_hp_threshold { weights.bloodlust_multiplier } else { 1.0 };

    details.offense += (state.enemy.max_hp as f64 - state.enemy.hp as f64)
        * weights.enemy_hp
        * checkmate_mult
        * bloodlust_mult
        * survival_multiplier;

    // NEW: Low Boss HP Reward
    if state.enemy.hp > 0 && state.enemy.hp <= 5 {
        details.offense += weights.low_boss_hp_reward * (6.0 - state.enemy.hp as f64);
    }

    if in_fire_panic || is_critical {
        // If boss is near death, don't dampen offense as much.
        let dampening = if (state.enemy.hp as f64) <= weights.checkmate_dampening_boss_hp_threshold {
            (weights.survival_only_multiplier * weights.panic_dampening_multiplier).min(1.0)
        } else {
            weights.survival_only_multiplier
        };
        details.offense *= dampening;
        details.situations *= dampening;
    }

    if state.enemy.hp <= 0 {
        details.offense += weights.boss_killing_blow_reward;
    }

    details.progression -= state.turn_count as f64 * weights.turn_penalty;
    details.progression -= history.len() as f64 * weights.step_penalty;

    // --- 2. Hazards ---
    let mut fire_rooms = HashSet::new();
    let mut water_count = 0;
    let mut fire_count = 0;

    for room in state.map.rooms.values() {
        let has_fire = room.hazards.contains(&HazardType::Fire);
        if has_fire {
            fire_rooms.insert(room.id);
            fire_count += room
                .hazards
                .iter()
                .filter(|h| **h == HazardType::Fire)
                .count();
        }
        water_count += room
            .hazards
            .iter()
            .filter(|h| **h == HazardType::Water)
            .count();
    }

    // Cubic Fire Penalty: Make firefighting > repairing
    // Non-Linear Fire Penalty based on Hull Risk (Inverse Power Function)
    // Use PROJECTED hull to feel the fear.
    // Less dampened: increase the risk multiplier as hull drops.
    let hull_risk_mult = (1.0 + (missing_hull_percent * weights.hull_risk_scaling_factor).powf(weights.hull_risk_scaling_exponent)).min(weights.hull_risk_max_mult);

    let mut fire_penalty = (fire_rooms.len() as f64).powf(weights.fire_exponent)
        * weights.fire_penalty_base
        * hull_risk_mult
        * hazard_multiplier;

    if state.is_resting {
        fire_penalty *= weights.rest_round_hazard_multiplier;
    }

    details.hazards -= fire_penalty;

    // Fire Token Penalty: Quadratic penalty for total fire tokens to reflect spreading risk.
    // Use hull_penalty_scaler for general urgency, but not doubled with hazard_multiplier.
    let mut fire_token_penalty =
        (fire_count as f64).powf(2.0) * weights.fire_token_penalty * hull_penalty_scaler;

    if state.is_resting {
        fire_token_penalty *= weights.rest_round_hazard_multiplier;
    }

    details.hazards -= fire_token_penalty;

    details.hazards -= water_count as f64 * weights.water_penalty * hull_penalty_scaler;

    // -- System Disabled --
    for room in state.map.rooms.values() {
        if let Some(sys) = room.system
            && matches!(
                sys,
                SystemType::Engine | SystemType::Cannons | SystemType::Bridge
            )
            && !room.hazards.is_empty()
        {
            let has_disabling_hazard = room.hazards.contains(&HazardType::Fire)
                || room.hazards.contains(&HazardType::Water);

            // Penalty for the system being currently disabled
            if has_disabling_hazard {
                details.hazards -= weights.system_disabled_penalty
                    * weights.system_importance_multiplier
                    * hull_penalty_scaler;
            }

            // New: Penalty for ANY hazard in a critical room (Danger Zone)
            // Even if not disabled yet (e.g. fire just started), we want to clear it ASAP.
            details.hazards -= weights.critical_system_hazard_penalty
                * (room.hazards.len() as f64)
                * weights.system_importance_multiplier
                * hull_penalty_scaler;

            // New: Checkmate System Bonus
            if room.hazards.is_empty() && state.enemy.hp <= 10 {
                details.hazards += weights.checkmate_system_bonus;
            }
        }
    }

    // -- Cargo Hull Repair Incentive --
    let urgency = (MAX_HULL as f64 - projected_hull as f64)
        .max(0.0)
        .powf(weights.cargo_repair_exponent);

    // Reward for being in Cargo scales with damage
    let mut dynamic_incentive = urgency * weights.cargo_repair_incentive;

    if state.is_resting {
        dynamic_incentive *= weights.rest_round_vitals_multiplier;
    }

    let mut cargo_id = None;
    for r in state.map.rooms.values() {
        if r.system == Some(SystemType::Cargo) {
            cargo_id = Some(r.id);
            break;
        }
    }
    if let Some(cid) = cargo_id {
        let mut t_set = HashSet::new();
        t_set.insert(cid);
        for p in state.players.values() {
            if p.room_id == cid {
                details.vitals += dynamic_incentive;
            }
            // New: Proximity Reward (Move towards Cargo when damaged)
            if urgency > 0.0 {
                let dist = distances.min_distance(p.room_id, &t_set);
                if dist != 999 {
                    details.vitals += (20.0 - dist as f64).max(0.0)
                        * weights.cargo_repair_proximity_reward
                        * urgency;
                }
            }
        }
    }

    // -- Hazard Proximity Setup --
    let mut hazardous_rooms = fire_rooms;
    for room in state.map.rooms.values() {
        if room.hazards.contains(&HazardType::Water) {
            hazardous_rooms.insert(room.id);
        }
    }

    // We calculated fire_count (tokens) earlier. Let's use that.
    let fire_urgency_mult = if fire_count > 0 {
        1.0 + (fire_count as f64 * weights.fire_urgency_mult)
    } else {
        1.0
    };

    // Track minimum distance to each hazard across all players (for global coverage reward)
    // Initialize with 999 (unreachable)
    let mut hazard_min_dists: HashMap<RoomId, u32> =
        hazardous_rooms.iter().map(|&id| (id, 999)).collect();

    // --- 3. Active Situations ---
    let negative_situations = state
        .active_situations
        .iter()
        .filter(|c| get_behavior(c.id).get_sentiment() == CardSentiment::Negative)
        .count();

    details.situations -= (negative_situations as f64).powf(weights.situation_exponent)
        * weights.active_situation_penalty
        * hazard_multiplier;

    for card in &state.active_situations {
        if card.id == CardId::Overheating {
            let mut engine_id = None;
            for r in state.map.rooms.values() {
                if r.system == Some(SystemType::Engine) {
                    engine_id = Some(r.id);
                    break;
                }
            }
            if let Some(eid) = engine_id {
                let count = state.players.values().filter(|p| p.room_id == eid).count();
                details.situations -=
                    count as f64 * weights.situation_exposure_penalty * hazard_multiplier;
            }
        }
    }

    // --- 4. Pending Threats (Telegraphs) ---
    let protected = state.shields_active || state.evasion_active;

    if let Some(attack) = &state.enemy.next_attack {
        let players_in_target = state
            .players
            .values()
            .filter(|p| p.room_id == attack.target_room)
            .count();

        let targets_system = attack.target_system.is_some();
        let is_severe = players_in_target > 0 || targets_system;

        if protected {
            let hull_urgency_mult =
                1.0 + (MAX_HULL as f64 - state.hull_integrity as f64).powf(1.2) / 10.0;
            if is_severe {
                details.threats +=
                    weights.threat_severe_reward * hull_urgency_mult * hazard_multiplier; // High value for mitigating a real threat
            } else {
                details.threats += weights.threat_mitigated_reward; // Small value for safety
            }

            // Defensive Posture Heuristic: Reward safety when critical
            details.threats += 5000.0 * missing_hull_percent * hazard_multiplier;
        } else {
            // Not protected - Apply penalties
            if players_in_target > 0 {
                details.threats -= players_in_target as f64
                    * weights.threat_player_penalty
                    * threat_mult
                    * hazard_multiplier;
            }

            if let Some(sys) = attack.target_system
                && matches!(
                    sys,
                    SystemType::Engine | SystemType::Cannons | SystemType::Bridge
                )
            {
                details.threats -= weights.threat_system_penalty * threat_mult * hazard_multiplier;
            }
            // General hull risk
            details.threats -= weights.threat_system_penalty
                * weights.threat_hull_risk_mult
                * threat_mult
                * hazard_multiplier;
        }
    } else {
        // No attack coming, but shields up? Wasted AP mostly, but small safety bonus
        if protected {
            details.threats -= weights.threat_shield_waste_penalty;
        }
    }

    // --- 5. Player Heuristics ---
    let mut cannon_rooms = HashSet::new();
    let mut kitchen_rooms = HashSet::new();
    let mut sickbay_rooms = HashSet::new();
    let mut nut_locations = HashSet::new();

    for room in state.map.rooms.values() {
        if let Some(sys) = &room.system {
            match sys {
                SystemType::Cannons => {
                    cannon_rooms.insert(room.id);
                }
                SystemType::Kitchen => {
                    kitchen_rooms.insert(room.id);
                }
                SystemType::Sickbay => {
                    sickbay_rooms.insert(room.id);
                }
                _ => {}
            }
        }
        if room.items.contains(&ItemType::Peppernut) {
            nut_locations.insert(room.id);
        }
    }

    // Pre-calculate ammo sources
    let mut ammo_sources = nut_locations;
    for k in &kitchen_rooms {
        ammo_sources.insert(*k);
    }

    for p in state.players.values() {
        if p.hp <= 0 {
            details.vitals -= weights.death_penalty;
            continue;
        }

        details.vitals += p.hp as f64 * weights.player_hp;
        details.logistics += p.ap as f64 * weights.ap_balance;

        let peppernuts = p
            .inventory
            .iter()
            .filter(|i| **i == ItemType::Peppernut)
            .count();
        let has_wheelbarrow = p.inventory.contains(&ItemType::Wheelbarrow);
        let has_extinguisher = p.inventory.contains(&ItemType::Extinguisher);
        let nut_cap = if has_wheelbarrow { 5 } else { 1 };

        // --- NEW: Station Keeping (Assigned Roles) ---
        let mut assigned_room = match p.id.as_str() {
            "P1" => find_room_with_system(state, SystemType::Kitchen),
            "P3" => find_room_with_system(state, SystemType::Bridge),
            "P4" => find_room_with_system(state, SystemType::Engine),
            "P5" | "P6" => find_room_with_system(state, SystemType::Cannons),
            _ => None, // P2 is a Roamer/Quartermaster
        };

        // --- NEW: Sentinel Reward ---
        if let Some(room) = state.map.rooms.get(&p.room_id)
            && let Some(sys) = room.system
        {
            if matches!(
                sys,
                SystemType::Cannons | SystemType::Engine | SystemType::Bridge
            ) && room.hazards.is_empty()
            {
                details.logistics += weights.sentinel_reward;
            }
        }

        // Dynamic Role Override: Critical Health (Seek Healing)
        if p.hp < 2 {
            assigned_room = None;
        }

        // Dynamic Role Override: Hazard Panic
        // If there is ANY hazard, only Gunners stick to their posts (unless hazard is in Cannons).
        // Everyone else becomes an emergency responder.
        let total_hazards = fire_count_total + water_count;
        if total_hazards > 0 {
            if matches!(p.id.as_str(), "P5" | "P6") {
                // Gunners only leave if it's really bad or hazard is IN the cannons
                if total_hazards >= weights.critical_fire_threshold
                    || find_room_with_system(state, SystemType::Cannons)
                        .and_then(|id| state.map.rooms.get(&id))
                        .is_some_and(|r| !r.hazards.is_empty())
                {
                    assigned_room = None;
                }
            } else {
                // Everyone else: Drop everything and help.
                assigned_room = None;
            }
        }

        if let Some(target) = assigned_room
            && p.room_id == target
        {
            details.logistics += weights.station_keeping_reward;
        }
        // Removed distance penalty to prevent interference with temporary tasks

        // -- Role: Healer --
        if p.hp < 3 {
            let urgency = (3 - p.hp).pow(2) as f64;
            let dist = distances.min_distance(p.room_id, &sickbay_rooms);
            if dist == 0 {
                details.vitals += weights.healing_reward * urgency;
            } else {
                details.vitals +=
                    (20.0 - dist as f64).max(0.0) * weights.sickbay_distance_factor * urgency;
            }
        }

        // -- Role: Gunner Logic --
        // P5/P6 are strictly Gunners via Station Keeping, but generic logic applies too.
        if peppernuts > 0 {
            let mut gunner_score = 0.0;
            let dist = distances.min_distance(p.room_id, &cannon_rooms);

            // Distance heuristic: Pull players with ammo to cannons
            // Note: Not multiplied by peppernuts to prevent hoarding abuse
            gunner_score += (weights.gunner_dist_range - dist as f64).max(0.0)
                * weights.gunner_distance_factor
                * (1.0 + weights.gunner_per_ammo_mult * peppernuts as f64);

            if dist == 0 {
                // At cannons: Reward quantity heavily to encourage stocking/shooting
                gunner_score +=
                    weights.gunner_per_ammo * peppernuts as f64 + weights.gunner_base_reward;
                // Bonus if Cannons system is working
                for &rid in &cannon_rooms {
                    if let Some(r) = state.map.rooms.get(&rid)
                        && !r.hazards.contains(&HazardType::Fire)
                    {
                        gunner_score += weights.gunner_working_bonus;
                    }
                }
            } else {
                // En route: Small reward per nut to encourage picking up, but not hoarding
                gunner_score +=
                    weights.gunner_per_ammo * weights.gunner_en_route_mult * peppernuts as f64;
            }

            if has_wheelbarrow && peppernuts < 5 {
                gunner_score *= weights.gunner_wheelbarrow_penalty;
            }
            details.logistics += gunner_score;
        }

        // -- Role: Emergency (Fire & Repair) & Global Coverage --
        // Unified logic: Go to hazards. Prioritize critical systems.
        let mut emergency_score = 0.0;
        let mut best_target_dist = 999;
        let mut is_critical_target = false;
        let mut best_target_is_fire = false;

        // Iterate all hazardous rooms to find best emergency target for THIS player
        // AND update global coverage stats.
        for &hid in &hazardous_rooms {
            let room = state.map.rooms.get(&hid).unwrap();

            // Calculate distance to this hazard for THIS player
            let mut t_set = HashSet::new();
            t_set.insert(hid);
            let d = distances.min_distance(p.room_id, &t_set);

            // Update Global Hazard Coverage (lowest distance wins)
            if let Some(min_d) = hazard_min_dists.get_mut(&hid)
                && d < *min_d
            {
                *min_d = d;
            }

            // --- Individual Role Logic ---
            let has_fire = room.hazards.contains(&HazardType::Fire);

            let is_critical_room = matches!(
                room.system,
                Some(SystemType::Engine) | Some(SystemType::Cannons) | Some(SystemType::Bridge)
            );

            if d < best_target_dist {
                best_target_dist = d;
                is_critical_target = is_critical_room;
                best_target_is_fire = has_fire;
            } else if d == best_target_dist {
                // Tie-breaker: Critical system wins
                if is_critical_room && !is_critical_target {
                    is_critical_target = true;
                    best_target_is_fire = has_fire;
                }
            }
        }

        if best_target_dist != 999 {
            // Panic Multiplier: Scale firefighting importance from 1x to 10x as damage increases.
            let panic_mult = 1.0 + (missing_hull_percent * weights.firefighter_panic_scaling);

            if best_target_dist == 0 {
                emergency_score += (weights.firefighter_base_reward / 2.0) * panic_mult * hazard_multiplier;
            } else {
                emergency_score += (20.0 - best_target_dist as f64).max(0.0)
                    * (weights.firefighter_distance_factor / 2.0)
                    * panic_mult
                    * hazard_multiplier;
            }

            // Critical Bonus (was "Targeted Repair")
            if is_critical_target {
                emergency_score += (20.0 - best_target_dist as f64).max(0.0)
                    * (weights.repair_proximity_reward / 2.0)
                    * hazard_multiplier;
            }

            if state.is_resting {
                emergency_score *= weights.rest_round_hazard_multiplier;
            }

            // Extinguisher Bonus
            if best_target_is_fire && has_extinguisher {
                emergency_score *= 1.5;
            }

            details.hazards += emergency_score;
        }

        // -- Role: Scavenger (Ammo Gathering) --
        // Encourage gathering if low on ammo. Stronger with Wheelbarrow.
        if peppernuts < nut_cap && state.enemy.hp > 0 {
            let mut scavenger_score = 0.0;

            if !ammo_sources.is_empty() {
                let d = distances.min_distance(p.room_id, &ammo_sources);
                scavenger_score += (20.0 - d as f64).max(0.0) * weights.scavenger_reward;
            }

            if has_wheelbarrow {
                scavenger_score *= weights.baker_wheelbarrow_mult;
            }
            details.logistics += scavenger_score;
        }
    }

    // --- Apply Global Hazard Coverage Reward ---
    for (_, min_d) in hazard_min_dists {
        if (min_d as f64) < weights.hazard_proximity_range {
            details.hazards += (weights.hazard_proximity_range - min_d as f64)
                * weights.hazard_proximity_reward
                * fire_urgency_mult
                * hazard_multiplier;
        }
    }

    // --- 7. Situation Solving (Precise) ---
    for card in &state.active_situations {
        if let Some(sol) = &card.solution
            && let Some(sys) = sol.target_system
        {
            let mut target_room = None;
            for r in state.map.rooms.values() {
                if r.system == Some(sys) {
                    target_room = Some(r.id);
                    break;
                }
            }
            if let Some(tid) = target_room {
                let mut qualified_distances = Vec::new();
                let mut all_distances = Vec::new();
                let required_item = sol.item_cost.as_ref();

                let mut t_set = HashSet::new();
                t_set.insert(tid);

                for p in state.players.values() {
                    if p.hp > 0 {
                        let d = distances.min_distance(p.room_id, &t_set);
                        all_distances.push(d);

                        // Check Item Requirement
                        let meets_req = match required_item {
                            Some(item) => p.inventory.contains(item),
                            None => true,
                        };

                        if meets_req {
                            qualified_distances.push(d);
                        }
                    }
                }
                qualified_distances.sort();
                all_distances.sort();

                if let Some(&best_q_d) = qualified_distances.first() {
                    let req_n = sol.required_players as usize;
                    let best_n_d = all_distances.get(req_n - 1).cloned().unwrap_or(999);

                    // Situation Importance: Boost reward for critical systems
                    let is_critical_sys = matches!(
                        sys,
                        SystemType::Engine | SystemType::Cannons | SystemType::Bridge
                    );
                    let mut importance_mult = if is_critical_sys {
                        weights.system_importance_multiplier * hull_penalty_scaler
                    } else {
                        survival_multiplier
                    };

                    // NEW: Blocking Situation Multiplier
                    // If the situation is negative, it's likely blocking or harming us.
                    if get_behavior(card.id).get_sentiment() == CardSentiment::Negative {
                        importance_mult *= weights.blocking_situation_multiplier;
                    }

                    if best_q_d == 0 && best_n_d == 0 {
                        details.situations += weights.solution_solver_reward * importance_mult;
                    } else {
                        // Pull the qualified person
                        details.situations += (20.0 - best_q_d as f64).max(0.0)
                            * weights.solution_distance_factor
                            * importance_mult;
                        // Pull the N-th person
                        if req_n > 1 && best_n_d != 999 {
                            details.situations += (20.0 - best_n_d as f64).max(0.0)
                                * weights.solution_distance_factor
                                * importance_mult;
                        }
                    }
                }

                // Logistics: If needed item is missing, reward getting it
                let someone_has_item = qualified_distances.iter().any(|&d| d != 999);
                if !someone_has_item && let Some(item) = required_item {
                    let mut sources = HashSet::new();
                    if *item == ItemType::Peppernut {
                        for &k in &ammo_sources {
                            sources.insert(k);
                        }
                    } else {
                        // Generic item search
                        for r in state.map.rooms.values() {
                            if r.items.contains(item) {
                                sources.insert(r.id);
                            }
                        }
                    }

                    let is_critical_sys = matches!(
                        sys,
                        SystemType::Engine | SystemType::Cannons | SystemType::Bridge
                    );
                    let importance_mult = if is_critical_sys {
                        weights.system_importance_multiplier * hull_penalty_scaler
                    } else {
                        survival_multiplier
                    };

                    if !sources.is_empty() {
                        let mut min_source_dist = 999;
                        for p in state.players.values() {
                            if p.hp > 0 {
                                let d = distances.min_distance(p.room_id, &sources);
                                if d < min_source_dist {
                                    min_source_dist = d;
                                }
                            }
                        }
                        if min_source_dist != 999 {
                            details.situations += (20.0 - min_source_dist as f64).max(0.0)
                                * weights.situation_logistics_reward
                                * importance_mult;
                        }
                    }
                }
            }
        }
    }

    // --- 8. Logistics (Ammo Stockpile) ---
    for room in state.map.rooms.values() {
        let nuts = room
            .items
            .iter()
            .filter(|i| **i == ItemType::Peppernut)
            .count();

        // General loose ammo reward
        details.logistics += nuts as f64 * weights.loose_ammo_reward;

        if room.system == Some(SystemType::Cannons) {
            // Massive reward for having ammo at Cannons
            details.logistics += nuts as f64 * weights.ammo_stockpile_reward;
        }
    }

    // --- 6. Trajectory Heuristics (Anti-Oscillation & Inaction) ---
    let mut player_moves: HashMap<PlayerId, Vec<(usize, u32)>> = HashMap::new();
    let mut last_action_by_player: HashMap<PlayerId, &GameAction> = HashMap::new();

    for (idx, item) in history.iter().enumerate() {
        let (pid, act) = *item;
        if matches!(act, GameAction::Shoot) && state.enemy.hp > 0 {
            let mut reward = weights.shooting_reward * survival_multiplier;
            if in_fire_panic || is_critical {
                let dampening = if (state.enemy.hp as f64) <= weights.checkmate_dampening_boss_hp_threshold {
                    (weights.survival_only_multiplier * weights.panic_dampening_multiplier).min(1.0)
                } else {
                    weights.survival_only_multiplier
                };
                reward *= dampening;
            }
            details.offense += reward;
        }

        if matches!(act, GameAction::Bake) {
            let mut nuts_in_room = 0;
            if let Some(room_id) = state.players.get(pid).map(|p| p.room_id) {
                if let Some(room) = state.map.rooms.get(&room_id) {
                    nuts_in_room = room.items.iter().filter(|i| **i == ItemType::Peppernut).count();
                }
            }
            if nuts_in_room < 10 {
                details.logistics += weights.bake_reward;
            }
        }

        // Inaction Penalty: Penalize repeated Pass/VoteReady by the same player
        if matches!(act, GameAction::Pass | GameAction::VoteReady { .. })
            && let Some(prev_act) = last_action_by_player.get(pid)
            && matches!(prev_act, GameAction::Pass | GameAction::VoteReady { .. })
        {
            details.anti_oscillation -= weights.inaction_penalty;
        }
        last_action_by_player.insert(pid.clone(), act);

        // Item Juggling Penalty: Penalize PickUp followed by Drop
        if let GameAction::Drop { item_index: _ } = act {
            let mut found_pickup = false;
            for prev_idx in (0..idx).rev() {
                let (prev_pid, prev_act) = history[prev_idx];
                if prev_pid != pid {
                    // Special case: Someone else threw an item to us.
                    // If we drop it immediately, it's juggling.
                    if let GameAction::Throw { target_player, .. } = prev_act {
                        if target_player == pid {
                            found_pickup = true;
                            break;
                        }
                    }
                    continue;
                }
                match prev_act {
                    GameAction::PickUp { item_type: _ } => {
                        found_pickup = true;
                        break;
                    }
                    GameAction::Move { .. }
                    | GameAction::Shoot
                    | GameAction::Throw { .. }
                    | GameAction::FirstAid { .. }
                    | GameAction::Revive { .. }
                    | GameAction::Repair
                    | GameAction::Extinguish
                    | GameAction::Interact
                    | GameAction::Bake
                    | GameAction::RaiseShields
                    | GameAction::EvasiveManeuvers
                    | GameAction::Lookout => {
                        // Useful action in between, not juggling
                        break;
                    }
                    _ => {}
                }
            }
            if found_pickup {
                details.anti_oscillation -= weights.item_juggling_penalty;
            }
        }

        if let GameAction::Move { to_room } = act {
            player_moves
                .entry(pid.clone())
                .or_default()
                .push((idx, *to_room));
        }
    }
    for (pid, moves) in player_moves {
        if moves.len() < 3 {
            continue;
        }
        for i in 0..moves.len() - 2 {
            let (_idx_a_start, room_a) = moves[i];
            let (idx_b, _room_b) = moves[i + 1];
            let (idx_a_return, room_a_return) = moves[i + 2];

            if room_a == room_a_return {
                let mut useful = false;
                for j in (idx_b + 1)..idx_a_return {
                    if let Some(item) = history.get(j) {
                        let (actor, action) = *item;
                        if actor == &pid {
                            match action {
                                GameAction::Move { .. } => {}
                                GameAction::Pass
                                | GameAction::VoteReady { .. }
                                | GameAction::Undo { .. }
                                | GameAction::Chat { .. } => {}
                                _ => {
                                    useful = true;
                                    break;
                                }
                            }
                        }
                        match action {
                            GameAction::Throw { target_player, .. }
                            | GameAction::Revive { target_player }
                            | GameAction::FirstAid { target_player } => {
                                if target_player == &pid {
                                    useful = true;
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                if !useful {
                    details.anti_oscillation -= weights.backtracking_penalty;
                }
            }
        }
    }

    details.total = details.vitals
        + details.hazards
        + details.offense
        + details.panic
        + details.logistics
        + details.situations
        + details.threats
        + details.progression
        + details.anti_oscillation;

    details
}

/// Calculates the score delta based on the transition from parent to current state.
fn score_transition(
    parent: &GameState,
    current: &GameState,
    weights: &BeamScoringWeights,
) -> ScoreDetails {
    let mut details = ScoreDetails::default();
    // Hull Delta Penalty: Penalize the ACT of losing hull
    let hull_loss = (parent.hull_integrity as f64 - current.hull_integrity as f64).max(0.0);
    details.vitals -= hull_loss * weights.hull_delta_penalty;

    // Situation Resolved Reward
    // If count decreased, we likely solved something.
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
        details.situations += weights.situation_resolved_reward;
    }

    if parent.enemy.hp > 0 && current.enemy.hp <= 0 {
        details.offense += weights.boss_killing_blow_reward;
    }

    details.total = details.vitals + details.situations + details.offense;
    details
}

/// Accumulator for trajectory scoring.
#[derive(Debug, Default, Clone)]
pub struct ScoreAccumulator {
    pub total_hull_integral: f64,
    pub total_hazard_integral: f64,
    pub total_enemy_hp_integral: f64,
    pub rounds_survived: u32,
    pub victory: bool,
}

impl ScoreAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_round_end(&mut self, state: &GameState) {
        self.rounds_survived = state.turn_count;
        self.total_hull_integral += state.hull_integrity as f64;
        self.total_enemy_hp_integral += state.enemy.hp as f64;
        let hazard_count: usize = state.map.rooms.values().map(|r| r.hazards.len()).sum();
        self.total_hazard_integral += hazard_count as f64;
    }

    pub fn finalize(&mut self, final_state: &GameState) -> f64 {
        if final_state.phase == GamePhase::Victory {
            self.victory = true;
        }

        let mut score = 0.0;
        if self.victory {
            score += 1_000_000.0;
        }
        score += self.total_hull_integral * 10.0;
        score -= self.total_hazard_integral * 5.0;
        score -= self.total_enemy_hp_integral * 20.0;
        if self.victory {
            score -= self.rounds_survived as f64 * 100.0;
        } else {
            score += self.rounds_survived as f64 * 100.0;
        }
        score
    }
}
