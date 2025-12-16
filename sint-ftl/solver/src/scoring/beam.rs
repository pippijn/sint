use super::ScoreDetails;
use serde::{Deserialize, Serialize};
use sint_core::logic::pathfinding::MapDistances;
use sint_core::logic::{cards::get_behavior, find_room_with_system};
use sint_core::small_map::SmallSet;
use sint_core::types::{
    AttackEffect, CardId, CardSentiment, GameAction, GamePhase, GameState, HazardType, ItemType,
    MAX_HULL, PlayerId, RoomId, SystemType,
};
use smallvec::SmallVec;

/// Hyperparameters for the scoring function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamScoringWeights {
    // Vital Stats
    pub hull_integrity: f64,
    pub hull_delta_penalty: f64,
    pub enemy_hp: f64,
    pub player_hp: f64,
    pub ap_balance: f64,

    // Systems
    pub system_health_reward: f64,
    pub system_broken_penalty: f64,

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
    pub ship_ammo_reward: f64,
    pub ship_ammo_cap: f64,

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

    pub fire_panic_threshold_base: f64,         // New
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

    pub survival_round_reward: f64, // New: Reward for living through rounds
    pub survival_step_reward: f64,  // New: Reward for living through steps

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

    pub panic_anti_oscillation_mult: f64, // New: Relax anti-oscillation in panic

    pub item_juggling_penalty: f64, // New: Penalty for picking up and dropping the same item

    // Multipliers & Ranges
    pub fire_urgency_mult: f64,
    pub hazard_proximity_range: f64,
    pub gunner_dist_range: f64,
    pub gunner_per_ammo_mult: f64,
    pub gunner_en_route_mult: f64,
    pub gunner_wheelbarrow_penalty: f64,
    pub baker_wheelbarrow_mult: f64,
    pub gunner_coordination_bonus: f64,

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

    pub system_ready_bonus: f64,
    pub wasted_defense_penalty: f64,
}

impl Default for BeamScoringWeights {
    fn default() -> Self {
        Self {
            hull_integrity: 20000.0,
            hull_delta_penalty: 10000.0,
            enemy_hp: 5000.0,
            player_hp: 1.0,
            ap_balance: 20.0,

            // Systems
            system_health_reward: 50.0,
            system_broken_penalty: 50000.0,

            // Hazards
            fire_penalty_base: 75000.0,
            fire_token_penalty: 10000.0,
            water_penalty: 20000.0,

            // Situations & Threats
            active_situation_penalty: 100000.0,
            threat_player_penalty: 20.0,
            threat_system_penalty: 200.0,
            death_penalty: 10000.0,

            // Roles
            station_keeping_reward: 10.0,
            gunner_base_reward: 2000.0,
            gunner_per_ammo: 1000.0,
            gunner_working_bonus: 500.0,
            gunner_distance_factor: 20.0,

            firefighter_base_reward: 30000.0,
            firefighter_distance_factor: 1000.0,

            healing_reward: 20.0,
            sickbay_distance_factor: 1.0,
            sentinel_reward: 200.0,

            // Anti-Oscillation
            backtracking_penalty: 100.0,
            commitment_bonus: 20.0,

            solution_solver_reward: 1000.0,
            solution_distance_factor: 2.0,
            situation_logistics_reward: 500.0,
            situation_resolved_reward: 25000.0,
            system_importance_multiplier: 15.0,
            boss_killing_blow_reward: 200000000.0,
            inaction_penalty: 50000.0,

            // Logistics
            ammo_stockpile_reward: 25000.0,
            loose_ammo_reward: 200.0,
            hazard_proximity_reward: 100.0,
            situation_exposure_penalty: 0.01,
            system_disabled_penalty: 75000.0,
            shooting_reward: 2000000.0,
            ship_ammo_reward: 500.0,
            ship_ammo_cap: 10.0,
            scavenger_reward: 200.0,
            repair_proximity_reward: 100.0,
            cargo_repair_incentive: 15000.0,
            cargo_repair_proximity_reward: 200.0,
            item_juggling_penalty: 10000.0,
            situation_exponent: 2.5,

            // Progression
            boss_level_reward: 200000.0,
            turn_penalty: 5000.0,
            step_penalty: 100.0,

            survival_round_reward: 1000.0,
            survival_step_reward: 1.0,

            checkmate_system_bonus: 10000.0,

            // Checkmate
            checkmate_threshold: 20.0,
            checkmate_multiplier: 250000.0,
            checkmate_max_mult: 2500000.0,

            // Critical State
            critical_hull_threshold: 12.0,
            critical_hull_penalty_base: 10000.0,
            critical_hull_penalty_per_hp: 1000.0,
            critical_fire_threshold: 1,
            critical_fire_penalty_per_token: 50.0,
            critical_system_hazard_penalty: 500.0,
            fire_in_critical_hull_penalty: 50000000.0,
            critical_survival_mult: 0.5,
            critical_threat_mult: 2.0,
            panic_anti_oscillation_mult: 0.1,

            // Exponents
            hull_exponent: 1.5,
            fire_exponent: 2.0,
            cargo_repair_exponent: 1.5,
            hull_risk_exponent: 1.2,
            panic_fire_exponent: 2.5,
            panic_hull_exponent: 2.0,
            checkmate_exponent: 2.0,
            hull_penalty_scaling: 1.1,
            projected_hull_panic_exponent: 2.5,

            fire_panic_threshold_base: 1.0,
            fire_panic_threshold_hull_scaling: 5.0,
            survival_only_multiplier: 0.5,

            // Multipliers
            fire_urgency_mult: 10.0,
            hazard_proximity_range: 20.0,
            gunner_dist_range: 20.0,
            gunner_per_ammo_mult: 0.5,
            gunner_en_route_mult: 0.2,
            gunner_wheelbarrow_penalty: 0.1,
            baker_wheelbarrow_mult: 3.0,
            gunner_coordination_bonus: 20000.0,

            threat_severe_reward: 5000.0,
            threat_mitigated_reward: 100.0,
            threat_hull_risk_mult: 0.5,
            threat_shield_waste_penalty: 0.01,

            rest_round_hazard_multiplier: 10.0,
            rest_round_vitals_multiplier: 10.0,

            victory_score: 1000000000.0,
            game_over_score: -1000000000.0,
            victory_hull_multiplier: 100000.0,
            fire_spread_projected_damage: 1.0,
            max_hazard_multiplier: 5.0,
            critical_survival_boss_hp_threshold: 5.0,
            hull_risk_scaling_factor: 10.0,
            hull_risk_scaling_exponent: 3.0,
            hull_risk_max_mult: 20.0,
            bloodlust_hp_threshold: 0.9,
            bloodlust_multiplier: 10.0,
            checkmate_dampening_boss_hp_threshold: 10.0,
            panic_dampening_multiplier: 20.0,
            firefighter_panic_scaling: 5.0,

            bake_reward: 1000.0,
            low_boss_hp_reward: 100000.0,
            blocking_situation_multiplier: 1000.0,

            system_ready_bonus: 2000.0,
            wasted_defense_penalty: 10000.0,
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

    // Meaningful Action Bonus: Reward productive actions
    if let Some((_pid, action)) = history.last()
        && matches!(
            action,
            GameAction::Extinguish
                | GameAction::Repair
                | GameAction::Shoot
                | GameAction::Bake
                | GameAction::RaiseShields
                | GameAction::EvasiveManeuvers
                | GameAction::FirstAid { .. }
                | GameAction::Revive { .. }
        )
    {
        details.logistics += 5000.0;
        details.total += 5000.0;
    }

    // Commitment Bonus: Reward moving towards hazards
    if let Some((last_pid, last_act)) = history.last()
        && matches!(last_act, GameAction::Move { .. })
        && let (Some(old_p), Some(new_p)) =
            (parent.players.get(last_pid), current.players.get(last_pid))
    {
        let old_room = old_p.room_id;
        let new_room = new_p.room_id;

        // Identify targets (Fires & Broken Systems)
        let mut target_rooms = SmallSet::new();
        for r in current.map.rooms.values() {
            if r.hazards.contains(&HazardType::Fire) || r.is_broken {
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
        details.vitals =
            weights.victory_score + (state.hull_integrity as f64 * weights.victory_hull_multiplier);
        details.total = details.vitals;
        return details;
    }
    // SUICIDE PREVENTION: Death must be worse than any living hell.
    // GRADIENT: Reward surviving as long as possible even if death is certain.
    if state.phase == GamePhase::GameOver || state.hull_integrity <= 0 {
        details.total = weights.game_over_score
            + (state.turn_count as f64 * weights.survival_round_reward)
            + (history.len() as f64 * weights.survival_step_reward);
        return details;
    }

    // --- ANALYSIS PASS: One pass over rooms to collect all metrics ---
    let mut rooms_on_fire = 0;
    let mut fire_spread_sources = 0;
    let mut fire_count_total = 0;
    let mut water_count_total = 0;
    let mut fire_rooms = SmallSet::new();
    let mut fire_penalty_raw = 0.0;
    let mut hazardous_rooms = SmallSet::new();
    let mut cannon_rooms = SmallSet::new();
    let mut kitchen_rooms = SmallSet::new();
    let mut sickbay_rooms = SmallSet::new();
    let mut nut_locations = SmallSet::new();
    let mut cargo_id = None;
    let mut engine_id = None;
    let mut total_nuts_in_rooms = 0;

    // Fast system lookup using fixed-size array
    let mut system_to_room: [Option<RoomId>; SystemType::COUNT] = [None; SystemType::COUNT];

    // Accumulators for scores that we can calculate early
    let mut system_health_reward_acc = 0.0;
    let mut broken_systems_penalty_acc = 0.0;
    let mut disabled_systems_penalty_acc = 0.0;
    let mut critical_hazards_penalty_acc = 0.0;
    let mut checkmate_system_bonus_acc = 0.0;
    let mut nuts_logistics_acc = 0.0;

    // Item Mapping for fast lookup
    let mut item_to_rooms: [SmallSet<RoomId>; ItemType::COUNT] = Default::default();

    for room in state.map.rooms.values() {
        let mut f_count = 0;
        let mut w_count = 0;
        for h in &room.hazards {
            match h {
                HazardType::Fire => f_count += 1,
                HazardType::Water => w_count += 1,
            }
        }

        if f_count > 0 {
            rooms_on_fire += 1;
            fire_count_total += f_count;
            fire_rooms.insert(room.id);
            if f_count >= 2 {
                fire_spread_sources += 1;
            }
            fire_penalty_raw += (f_count as f64).powf(weights.fire_exponent);
        }
        water_count_total += w_count;

        if f_count > 0 || w_count > 0 || room.is_broken {
            hazardous_rooms.insert(room.id);
        }

        if let Some(sys) = room.system {
            system_to_room[sys.as_u32() as usize] = Some(room.id);
            system_health_reward_acc += room.system_health as f64;

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
                SystemType::Cargo => cargo_id = Some(room.id),
                SystemType::Engine => engine_id = Some(room.id),
                _ => {}
            }

            if room.is_broken {
                broken_systems_penalty_acc += weights.system_importance_multiplier;
            }

            if matches!(
                sys,
                SystemType::Engine | SystemType::Cannons | SystemType::Bridge
            ) {
                let is_blocked = state.active_situations.iter().any(|card| {
                    card.solution
                        .as_ref()
                        .map(|s| s.target_system == Some(sys))
                        .unwrap_or(false)
                });
                if room.is_broken || f_count > 0 || w_count > 0 || is_blocked {
                    disabled_systems_penalty_acc += weights.system_importance_multiplier;
                }
                if !room.hazards.is_empty() {
                    critical_hazards_penalty_acc +=
                        (room.hazards.len() as f64) * weights.system_importance_multiplier;
                }
            }

            if room.hazards.is_empty() && state.enemy.hp <= 10 {
                checkmate_system_bonus_acc += weights.checkmate_system_bonus;
            }
        }

        for item in &room.items {
            item_to_rooms[item.as_usize()].insert(room.id);
        }

        let mut room_nuts = 0;
        for item in &room.items {
            if *item == ItemType::Peppernut {
                room_nuts += 1;
            }
        }

        if room_nuts > 0 {
            total_nuts_in_rooms += room_nuts;
            nut_locations.insert(room.id);
            nuts_logistics_acc += room_nuts as f64 * weights.loose_ammo_reward;
            if room.system == Some(SystemType::Cannons) {
                nuts_logistics_acc += room_nuts as f64 * weights.ammo_stockpile_reward;
            }
        }
    }

    details.logistics += nuts_logistics_acc;

    // PROJECTED DAMAGE: One hull damage per room on fire at end of round.
    let fire_damage = rooms_on_fire;

    let mut telegraphed_hull_damage = 0;
    if let Some(attack) = &state.enemy.next_attack
        && !state.shields_active
        && !state.evasion_active
    {
        match attack.effect {
            AttackEffect::Fireball => {
                // Fireball deals 1 damage per hit.
                // We should also account for situations that increase attack count.
                let mut count = 1;
                for card in &state.active_situations {
                    let behavior = get_behavior(card.id);
                    let c = behavior.get_enemy_attack_count(state);
                    if c > count {
                        count = c;
                    }
                }
                telegraphed_hull_damage += count as i32;
            }
            AttackEffect::Special(_) => {
                // Some special attacks might deal damage, but we'll stick to Fireball for now
                // as it's the most common direct damage.
            }
            _ => {}
        }
    }

    let projected_hull = state.hull_integrity
        - fire_damage as i32
        - telegraphed_hull_damage
        - (fire_spread_sources as f64 * weights.fire_spread_projected_damage) as i32;
    // If fires will kill us this round, apply massive penalty but don't prune yet.
    // This allows "death or glory" plays to win before the fire damage applies.
    // GRADIENT: Use a slightly less severe penalty for projected death than actual death.
    if projected_hull <= 0 && state.phase != GamePhase::Victory {
        details.panic += weights.game_over_score * 0.95
            + (state.turn_count as f64 * weights.survival_round_reward)
            + (history.len() as f64 * weights.survival_step_reward);
    }

    // Use PROJECTED hull for scaling
    let missing_hull_percent =
        ((MAX_HULL as f64 - projected_hull as f64) / MAX_HULL as f64).max(0.0_f64);

    // Continuous Multipliers:
    // survival_multiplier: 1.0 at full health, down to critical_survival_mult at 0 health.
    let survival_multiplier = 1.0 - (missing_hull_percent * (1.0 - weights.critical_survival_mult));

    // Offensive Survival Multiplier: Be more aggressive when boss is low
    let offense_survival_multiplier = if state.enemy.hp <= 10 {
        1.0
    } else {
        survival_multiplier
    };

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

        // EXTRA BUMP for 1 HP - The "Just Kill It" factor
        if state.enemy.hp == 1 {
            checkmate_mult *= 100.0; // Massive boost for final HP
        }

        // Survival Override: If we are in critical condition, dampen bloodlust.
        // Finishing the boss IS survival, but only if it happens SOON.
        if is_critical {
            if projected_hull <= 0 {
                // SURVIVAL SUPREMACY: Death is certain, but allow a "Death or Glory" attempt
                // if the boss is extremely low.
                if state.enemy.hp <= 5 {
                    checkmate_mult *= 0.1;
                } else {
                    checkmate_mult = 0.0;
                }
            } else if projected_hull <= 2 {
                // NEAR DEATH: Only allow checkmate if boss is 1-2 HP.
                if state.enemy.hp > 2 {
                    checkmate_mult *= 0.01;
                } else {
                    checkmate_mult *= 0.5;
                }
            } else if (state.enemy.hp as f64) > 2.0 {
                let survival_factor =
                    (projected_hull as f64 / weights.critical_hull_threshold).clamp(0.2, 1.0);
                checkmate_mult *= survival_factor;

                if (state.enemy.hp as f64) > weights.critical_survival_boss_hp_threshold {
                    checkmate_mult *= weights.survival_only_multiplier;
                }
            } else {
                // Boss is at 1-2 HP and we are NOT dead yet. GO FOR IT.
                // But scale by health to ensure we don't start next boss at 1 HP.
                let survival_factor = (projected_hull as f64 / MAX_HULL as f64).clamp(0.1, 1.0);
                checkmate_mult *= 1.5 * survival_factor;
            }
        }
    }

    // --- Critical State Heuristic ---
    // Override everything if we are about to die.
    // Survival is absolute. Do not reduce panic based on enemy health.

    if is_critical {
        let mut panic_penalty = weights.critical_hull_penalty_base;
        let deficit = (weights.critical_hull_threshold + 1.0 - projected_hull as f64).max(0.0_f64);
        panic_penalty += deficit.powf(weights.panic_hull_exponent)
            * weights.critical_hull_penalty_per_hp
            * hull_penalty_scaler;

        // New: If hull is critical AND there is any fire, add an extreme penalty.
        // This prevents the AI from ignoring "small" fires when it's one hit from death.
        if rooms_on_fire > 0 {
            let fire_panic_factor = (rooms_on_fire as f64).powf(weights.panic_fire_exponent);
            panic_penalty +=
                weights.fire_in_critical_hull_penalty * fire_panic_factor * hull_penalty_scaler;
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
        + (state.hull_integrity as f64 / MAX_HULL as f64)
            * weights.fire_panic_threshold_hull_scaling;
    let in_fire_panic = rooms_on_fire as f64 > fire_panic_threshold;

    if in_fire_panic || is_critical {
        // Survival Mode: If we are about to die or overwhelmed by fire,
        // focus on survival, but keep logistics active to ensure we have tools/ammo.
        let dampening = if (state.enemy.hp as f64) <= 5.0 {
            // Boss is almost dead! Victory is the best survival.
            2.0
        } else if (state.enemy.hp as f64) <= weights.checkmate_dampening_boss_hp_threshold {
            // Boss is low, but not dead.
            (weights.survival_only_multiplier * weights.panic_dampening_multiplier * 2.0).min(1.5)
        } else {
            weights.survival_only_multiplier
        };
        details.offense *= dampening;
        details.situations *= dampening;
        details.progression *= dampening;
        // Do NOT dampen logistics - we need ammo to end the threat!
    }

    // --- 1. Vital Stats & Progression ---
    // Non-linear Hull Penalty: Penalize missing hull exponentially (Square of missing health)
    // Use PROJECTED hull
    let missing_hull = (MAX_HULL as f64 - projected_hull as f64).max(0.0_f64);
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
    let bloodlust_mult = if enemy_hp_percent < weights.bloodlust_hp_threshold {
        weights.bloodlust_multiplier
    } else {
        1.0
    };

    details.offense += (state.enemy.max_hp as f64 - state.enemy.hp as f64)
        * weights.enemy_hp
        * checkmate_mult
        * bloodlust_mult
        * offense_survival_multiplier;

    // NEW: Low Boss HP Reward
    if state.enemy.hp > 0 && state.enemy.hp <= 5 {
        details.offense += weights.low_boss_hp_reward * (6.0 - state.enemy.hp as f64);
    }

    if state.enemy.hp <= 0 {
        details.offense += weights.boss_killing_blow_reward;
    }

    details.progression -= state.turn_count as f64 * weights.turn_penalty;
    details.progression -= history.len() as f64 * weights.step_penalty;

    // --- 1. Vital Stats & Progression ---
    // Cubic Fire Penalty: Make firefighting > repairing
    // Non-Linear Fire Penalty based on Hull Risk (Inverse Power Function)
    // Use PROJECTED hull to feel the fear.
    let hull_risk_mult = (1.0
        + (missing_hull_percent * weights.hull_risk_scaling_factor)
            .powf(weights.hull_risk_scaling_exponent))
    .min(weights.hull_risk_max_mult);

    let mut fire_penalty = fire_penalty_raw * weights.fire_penalty_base;
    // Also add room count penalty to encourage clearing rooms completely
    fire_penalty +=
        (rooms_on_fire as f64).powf(weights.fire_exponent) * weights.fire_penalty_base;

    fire_penalty *= hull_risk_mult * hazard_multiplier;

    if state.is_resting {
        fire_penalty *= weights.rest_round_hazard_multiplier;
    }

    details.hazards -= fire_penalty;

    // Fire Token Penalty: Quadratic penalty for total fire tokens to reflect spreading risk.
    let mut fire_token_penalty =
        (fire_count_total as f64).powf(2.0) * weights.fire_token_penalty * hull_penalty_scaler;

    if state.is_resting {
        fire_token_penalty *= weights.rest_round_hazard_multiplier;
    }

    details.hazards -= fire_token_penalty;

    details.hazards -= water_count_total as f64 * weights.water_penalty * hull_penalty_scaler;

    // -- System Penalties (using pre-calculated accumulators) --
    details.vitals += system_health_reward_acc * weights.system_health_reward;
    details.hazards -=
        broken_systems_penalty_acc * weights.system_broken_penalty * hull_penalty_scaler;
    details.hazards -=
        disabled_systems_penalty_acc * weights.system_disabled_penalty * hull_penalty_scaler;
    details.hazards -=
        critical_hazards_penalty_acc * weights.critical_system_hazard_penalty * hull_penalty_scaler;
    details.hazards += checkmate_system_bonus_acc;

    // -- Cargo Hull Repair Incentive --
    let urgency = (MAX_HULL as f64 - projected_hull as f64)
        .max(0.0_f64)
        .powf(weights.cargo_repair_exponent);

    // Reward for being in Cargo scales with damage
    let mut dynamic_incentive = urgency * weights.cargo_repair_incentive;

    if state.is_resting {
        dynamic_incentive *= weights.rest_round_vitals_multiplier;
    }

    if let Some(cid) = cargo_id {
        let mut t_set = SmallSet::new();
        t_set.insert(cid);
        for p in state.players.values() {
            if p.room_id == cid {
                details.vitals += dynamic_incentive;
            }
            // New: Proximity Reward (Move towards Cargo when damaged)
            if urgency > 0.0 {
                let dist = distances.min_distance(p.room_id, &t_set);
                if dist != 999 {
                    details.vitals += (20.0 - dist as f64).max(0.0_f64)
                        * weights.cargo_repair_proximity_reward
                        * urgency;
                }
            }
        }
    }

    // -- Hazard Proximity Setup --
    // We calculated fire_count_total (tokens) earlier. Let's use that.
    let fire_urgency_mult = if fire_count_total > 0 {
        1.0 + (fire_count_total as f64 * weights.fire_urgency_mult)
    } else {
        1.0
    };

    // Track minimum distance to each hazard across all players (for global coverage reward)
    // Using a SmallVec to keep allocations on the stack
    let mut hazard_min_dists: SmallVec<[(RoomId, u32); 16]> = hazardous_rooms.iter().map(|id| (id, 999)).collect();

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
            .filter(|p| Some(p.room_id) == attack.target_room)
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
            details.threats -= weights.wasted_defense_penalty;
        }
    }

    // Pre-calculate ammo sources
    let mut ammo_sources = nut_locations;
    for k in &kitchen_rooms {
        ammo_sources.insert(k);
    }

    // -- Role: Gunner Logic --
    let mut players_at_cannons_with_ammo = 0;
    for p in state.players.values() {
        if p.hp > 0 && p.inventory.contains(&ItemType::Peppernut) {
            let dist = distances.min_distance(p.room_id, &cannon_rooms);
            if dist == 0 {
                players_at_cannons_with_ammo += 1;
            }
        }
    }
    if players_at_cannons_with_ammo > 1 {
        details.logistics +=
            (players_at_cannons_with_ammo - 1) as f64 * weights.gunner_coordination_bonus;
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
            && matches!(
                sys,
                SystemType::Cannons | SystemType::Engine | SystemType::Bridge
            )
            && room.hazards.is_empty()
        {
            details.logistics += weights.sentinel_reward;
        }

        // --- NEW: System Ready Bonus ---
        if let Some(room) = state.map.rooms.get(&p.room_id)
            && let Some(sys) = room.system
            && !room.is_broken
            && room.hazards.is_empty()
        {
            let is_ready = match sys {
                SystemType::Cannons => p.inventory.contains(&ItemType::Peppernut),
                SystemType::Cargo => true, // Can always repair hull if no hazards
                SystemType::Engine => p.ap >= 2,
                SystemType::Bridge => p.ap >= 2,
                SystemType::Kitchen => true,
                SystemType::Sickbay => true, // Could check if anyone needs healing but being there is good
                _ => true,
            };

            if is_ready {
                details.logistics += weights.system_ready_bonus;
            }
        }

        // Dynamic Role Override: Critical Health (Seek Healing)
        if p.hp < 2 {
            assigned_room = None;
        }

        // Dynamic Role Override: Hazard Panic
        // If there is ANY hazard, only Gunners stick to their posts (unless hazard is in Cannons).
        // Everyone else becomes an emergency responder.
        let total_hazards = fire_count_total + water_count_total;
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
                    (20.0 - dist as f64).max(0.0_f64) * weights.sickbay_distance_factor * urgency;
            }
        }

        // -- Role: Gunner Logic --
        // P5/P6 are strictly Gunners via Station Keeping, but generic logic applies too.
        if peppernuts > 0 {
            let mut gunner_score = 0.0;
            let dist = distances.min_distance(p.room_id, &cannon_rooms);

            // Check if Cannons are operational
            let cannon_operational = cannon_rooms.iter().any(|rid| {
                if let Some(r) = state.map.rooms.get(&rid) {
                    let is_blocked = state.active_situations.iter().any(|card| {
                        card.solution
                            .as_ref()
                            .map(|s| s.target_system == Some(SystemType::Cannons))
                            .unwrap_or(false)
                    });
                    !r.is_broken
                        && !r.hazards.contains(&HazardType::Fire)
                        && !r.hazards.contains(&HazardType::Water)
                        && !is_blocked
                } else {
                    false
                }
            });

            // Distance heuristic: Pull players with ammo to cannons
            // Note: Not multiplied by peppernuts to prevent hoarding abuse
            gunner_score += (weights.gunner_dist_range - dist as f64).max(0.0_f64)
                * weights.gunner_distance_factor
                * (1.0 + weights.gunner_per_ammo_mult * peppernuts as f64);

            if dist == 0 {
                // At cannons: Reward quantity heavily to encourage stocking/shooting
                gunner_score +=
                    weights.gunner_per_ammo * peppernuts as f64 + weights.gunner_base_reward;
                // Bonus if Cannons system is working
                if cannon_operational {
                    gunner_score += weights.gunner_working_bonus;
                }
            } else {
                // En route: Small reward per nut to encourage picking up, but not hoarding
                gunner_score +=
                    weights.gunner_per_ammo * weights.gunner_en_route_mult * peppernuts as f64;
            }

            if has_wheelbarrow && peppernuts < 5 {
                gunner_score *= weights.gunner_wheelbarrow_penalty;
            }

            if !cannon_operational {
                gunner_score *= 0.1; // Drastically reduce value if cannon can't fire
            }

            details.logistics += gunner_score;

            // NEW: Potential Damage (Static Reward for holding ammo)
            // Pulled towards cannons if holding nuts
            let dist = distances.min_distance(p.room_id, &cannon_rooms);
            let mut pot_damage = (peppernuts as f64 * 4000.0) / (dist as f64 + 1.0);

            // Scale with checkmate and bloodlust to ensure we don't prune shooting paths
            pot_damage *= checkmate_mult * bloodlust_mult;

            // Critical boost for 1 HP
            if state.enemy.hp == 1 {
                pot_damage *= 10.0;
            }

            if !cannon_operational {
                pot_damage = 0.0; // No potential damage if cannon is jammed/broken
            }

            details.offense += pot_damage;
        }

        // -- Role: Emergency (Fire & Repair) & Global Coverage --
        // Unified logic: Go to hazards. Prioritize critical systems.
        let mut emergency_score = 0.0;
        let mut best_target_dist = 999;
        let mut is_critical_target = false;
        let mut best_target_is_fire = false;

        // Iterate all hazardous rooms to find best emergency target for THIS player
        // AND update global coverage stats.
        for hid in &hazardous_rooms {
            let room = state.map.rooms.get(&hid).unwrap();

            // Calculate distance to this hazard for THIS player
            let mut t_set = SmallSet::new();
            t_set.insert(hid);
            let d = distances.min_distance(p.room_id, &t_set);

            // Update Global Hazard Coverage (lowest distance wins)
            for (hid_global, min_d) in &mut hazard_min_dists {
                if *hid_global == hid && d < *min_d {
                    *min_d = d;
                }
            }

            // --- Individual Role Logic ---
            let has_fire = room.hazards.contains(&HazardType::Fire);
            let is_broken = room.is_broken;

            let is_critical_room = matches!(
                room.system,
                Some(SystemType::Engine) | Some(SystemType::Cannons) | Some(SystemType::Bridge)
            );

            if d < best_target_dist {
                best_target_dist = d;
                is_critical_target = is_critical_room;
                best_target_is_fire = has_fire;
            } else if d == best_target_dist {
                // Tie-breaker: Broken system or Critical system wins
                if (is_broken || is_critical_room) && !is_critical_target {
                    is_critical_target = true;
                    best_target_is_fire = has_fire;
                }
            }
        }

        if best_target_dist != 999 {
            // Panic Multiplier: Scale firefighting importance from 1x to 10x as damage increases.
            let panic_mult = 1.0 + (missing_hull_percent * weights.firefighter_panic_scaling);

            if best_target_dist == 0 {
                emergency_score +=
                    (weights.firefighter_base_reward / 2.0) * panic_mult * hazard_multiplier;
            } else {
                emergency_score += (20.0 - best_target_dist as f64).max(0.0_f64)
                    * (weights.firefighter_distance_factor / 2.0)
                    * panic_mult
                    * hazard_multiplier;
            }

            // Critical Bonus (was "Targeted Repair")
            if is_critical_target {
                emergency_score += (20.0 - best_target_dist as f64).max(0.0_f64)
                    * (weights.repair_proximity_reward / 2.0)
                    * hazard_multiplier;
            }

            if state.is_resting {
                emergency_score *= weights.rest_round_hazard_multiplier;
            }

            // Extinguisher Bonus
            if best_target_is_fire && has_extinguisher {
                emergency_score *= 2.0;
            }

            details.hazards += emergency_score;
        }

        // -- Role: Scavenger (Ammo Gathering) --
        // Encourage gathering if low on ammo. Stronger with Wheelbarrow.
        if peppernuts < nut_cap && state.enemy.hp > 0 {
            let mut scavenger_score = 0.0;

            if !ammo_sources.is_empty() {
                let d = distances.min_distance(p.room_id, &ammo_sources);
                scavenger_score += (20.0 - d as f64).max(0.0_f64) * weights.scavenger_reward;
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
            let target_room = system_to_room[sys.as_u32() as usize];
            if let Some(tid) = target_room {
                let mut qualified_distances: SmallVec<[u32; 8]> = SmallVec::new();
                let mut all_distances: SmallVec<[u32; 8]> = SmallVec::new();
                let required_item = sol.item_cost.as_ref();

                let mut t_set = SmallSet::new();
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
                        details.situations += (20.0_f64 - best_q_d as f64).max(0.0_f64)
                            * weights.solution_distance_factor
                            * importance_mult;
                        // Pull the N-th person
                        if req_n > 1 && best_n_d != 999 {
                            details.situations += (20.0_f64 - best_n_d as f64).max(0.0_f64)
                                * weights.solution_distance_factor
                                * importance_mult;
                        }
                    }
                }

                // Logistics: If needed item is missing, reward getting it
                let someone_has_item = qualified_distances.iter().any(|&d| d != 999);
                if !someone_has_item && let Some(item) = required_item {
                    let mut sources = SmallSet::new();
                    if *item == ItemType::Peppernut {
                        for k in &ammo_sources {
                            sources.insert(k);
                        }
                    } else {
                        // Generic item search using pre-calculated mapping
                        sources = item_to_rooms[item.as_usize()].clone();
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
                            details.situations += (20.0 - min_source_dist as f64).max(0.0_f64)
                                * weights.situation_logistics_reward
                                * importance_mult;
                        }
                    }
                }
            }
        }
    }

    // --- 8. Logistics (Ammo Stockpile) ---
    let mut total_nuts = total_nuts_in_rooms;

    // Include player inventory in total nuts
    for p in state.players.values() {
        total_nuts += p
            .inventory
            .iter()
            .filter(|i| **i == ItemType::Peppernut)
            .count();
    }

    // Ship-wide ammo reward (Capped to prevent infinite hoarding)
    details.logistics += (total_nuts as f64).min(weights.ship_ammo_cap) * weights.ship_ammo_reward;

    // Kitchen Proximity: Pull players towards kitchen if ammo is low
    if total_nuts < 10
        && let Some(kitchen_id) = find_room_with_system(state, SystemType::Kitchen)
    {
        let mut k_set = SmallSet::new();
        k_set.insert(kitchen_id);
        for p in state.players.values() {
            let d = distances.min_distance(p.room_id, &k_set);
            details.logistics += (20.0 - d as f64).max(0.0_f64) * 100.0;
        }
    }

    // --- 6. Trajectory Heuristics (Anti-Oscillation & Inaction) ---
    // Using fixed-size arrays to avoid heap allocations
    let mut player_moves: [SmallVec<[(usize, u32); 8]>; 8] = Default::default();
    let mut item_throw_count = 0;

    let player_to_idx = |pid: &PlayerId| -> usize {
        if pid.starts_with('P') {
            pid[1..].parse::<usize>().unwrap_or(0).saturating_sub(1).min(7)
        } else {
            0
        }
    };

    for (idx, item) in history.iter().enumerate() {
        let (pid, act) = *item;
        let p_idx = player_to_idx(pid);
        if matches!(act, GameAction::Shoot) && state.enemy.hp >= 0 {
            let mut reward = weights.shooting_reward * offense_survival_multiplier;

            // Finishing Blow Incentive: Massive reward for a successful kill in the trajectory
            if state.enemy.hp <= 0 {
                reward += weights.boss_killing_blow_reward;
            }

            if in_fire_panic || is_critical {
                let dampening = if (state.enemy.hp as f64)
                    <= weights.checkmate_dampening_boss_hp_threshold
                {
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
            if let Some(room_id) = state.players.get(pid).map(|p| p.room_id)
                && let Some(room) = state.map.rooms.get(&room_id)
            {
                nuts_in_room = room
                    .items
                    .iter()
                    .filter(|i| **i == ItemType::Peppernut)
                    .count();
            }

            // Only reward baking if we are below both room and ship-wide caps
            // AND we don't already have enough nuts to kill the boss
            if nuts_in_room < 10
                && (total_nuts as f64) < weights.ship_ammo_cap
                && (total_nuts as i32) < state.enemy.hp
            {
                let b_reward = weights.bake_reward;
                details.logistics += b_reward;
            }
        }

        if let GameAction::Throw { target_player, .. } = act {
            item_throw_count += 1;
            // Penalize excessive throwing (juggling)
            if item_throw_count > 2 {
                let mut penalty = 1000.0 * (item_throw_count as f64);
                if in_fire_panic || is_critical {
                    penalty *= weights.panic_anti_oscillation_mult;
                }
                details.anti_oscillation -= penalty;
            }

            // Reward throwing ammo to someone closer to cannons
            if let Some(target_p) = state.players.get(target_player)
                && let Some(our_p) = state.players.get(pid)
            {
                let our_dist = distances.min_distance(our_p.room_id, &cannon_rooms);
                let target_dist = distances.min_distance(target_p.room_id, &cannon_rooms);

                if target_dist < our_dist {
                    details.logistics += 5000.0; // Significant teamwork reward
                }
            }
        }

        // Item Juggling Penalty: Penalize PickUp followed by Drop
        if let GameAction::Drop { item_index: _ } = act {
            let mut found_pickup = false;
            for prev_idx in (0..idx).rev() {
                let (prev_pid, prev_act) = history[prev_idx];
                if prev_pid != pid {
                    // Special case: Someone else threw an item to us.
                    // If we drop it immediately, it's juggling.
                    if let GameAction::Throw { target_player, .. } = prev_act
                        && target_player == pid
                    {
                        found_pickup = true;
                        break;
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
                details.logistics -= weights.item_juggling_penalty;
            }
        }

        if let GameAction::Move { to_room } = act {
            player_moves[p_idx].push((idx, *to_room));
        }
    }

    // Inaction Penalty: Penalize if NO player did anything productive in this batch

    let batch_is_productive = history.iter().any(|(_actor, act)| {
        !matches!(
            act,
            GameAction::Pass | GameAction::VoteReady { .. } | GameAction::Chat { .. }
        )
    });

    if !batch_is_productive && !history.is_empty() {
        details.anti_oscillation -= weights.inaction_penalty;
    }

    for (p_idx, moves) in player_moves.iter().enumerate() {
        let moves_slice: &[(usize, u32)] = moves.as_slice();
        if moves_slice.len() < 3 {
            continue;
        }
        let pid_str = format!("P{}", p_idx + 1);
        for i in 0..moves_slice.len() - 2 {
            let (_idx_a_start, room_a) = moves_slice[i];
            let (idx_b, _room_b) = moves_slice[i + 1];
            let (idx_a_return, room_a_return) = moves_slice[i + 2];

            if room_a == room_a_return {
                let mut useful = false;
                for j in (idx_b + 1)..idx_a_return {
                    if let Some(item) = history.get(j) {
                        let (actor, action) = *item;
                        if actor == &pid_str {
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
                                if target_player == &pid_str {
                                    useful = true;
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                if !useful {
                    let mut penalty = weights.backtracking_penalty;
                    if in_fire_panic || is_critical {
                        penalty *= weights.panic_anti_oscillation_mult;
                    }
                    details.anti_oscillation -= penalty;
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
    let hull_loss = (parent.hull_integrity as f64 - current.hull_integrity as f64).max(0.0_f64);
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
        score -= self.rounds_survived as f64 * 100.0;
        score
    }
}
