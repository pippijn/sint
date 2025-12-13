use sint_core::logic::{cards::get_behavior, find_room_with_system};
use sint_core::types::{
    CardId, CardSentiment, GameAction, GamePhase, GameState, HazardType, ItemType, MAX_HULL,
    PlayerId, RoomId, SystemType,
};
use std::collections::{HashMap, HashSet, VecDeque};

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

    // Anti-Oscillation
    pub backtracking_penalty: f64,
    pub commitment_bonus: f64, // New: Reward for continuing a task

    // Situation Solving
    pub solution_solver_reward: f64,
    pub solution_distance_factor: f64,
    pub situation_logistics_reward: f64, // New: Reward for getting items needed for situations
    pub situation_resolved_reward: f64,  // New: Reward for the act of solving
    pub system_importance_multiplier: f64, // New: Multiplier for critical systems

    // Logistics
    pub ammo_stockpile_reward: f64,
    pub loose_ammo_reward: f64,
    pub hazard_proximity_reward: f64,
    pub situation_exposure_penalty: f64,
    pub system_disabled_penalty: f64,
    pub shooting_reward: f64,

    pub scavenger_reward: f64,
    pub repair_proximity_reward: f64,
    pub cargo_repair_incentive: f64,
    pub cargo_repair_proximity_reward: f64, // New: Reward for moving towards Cargo

    // Progression
    pub boss_level_reward: f64,
    pub turn_penalty: f64,
    pub step_penalty: f64,

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
    pub critical_survival_mult: f64,
    pub critical_threat_mult: f64,

    // Exponents (Non-Linearity)
    pub hull_exponent: f64,
    pub fire_exponent: f64,
    pub cargo_repair_exponent: f64,
    pub hull_risk_exponent: f64,
    pub panic_fire_exponent: f64,  // New
    pub panic_hull_exponent: f64,  // New
    pub checkmate_exponent: f64,   // New
    pub hull_penalty_scaling: f64, // New

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
}

impl Default for BeamScoringWeights {
    fn default() -> Self {
        Self {
            hull_integrity: 8000.0,     // Reduced from 10000.0 to accept more risk
            hull_delta_penalty: 5000.0, // Increased from 2000.0 to penalize damage more
            enemy_hp: 25000.0,          // High priority: KILL THE BOSS (Increased from 15000.0)
            player_hp: 200.0,
            ap_balance: 100.0, // High value on AP = Don't waste turns

            // Hazards - significantly increased penalties
            fire_penalty_base: 10000.0, // Reduced from 15000.0
            water_penalty: 1000.0,

            // Situations & Threats
            active_situation_penalty: 80000.0, // WAS 50000.0. Situations kill runs.
            threat_player_penalty: 100.0,      // Reduced to allow calculated risks
            threat_system_penalty: 2000.0,
            death_penalty: 50000.0,

            // Roles
            station_keeping_reward: 2000.0, // Reduced from 3000.0 to encourage flexibility
            gunner_base_reward: 600.0,      // STAY AT THE CANNONS
            gunner_per_ammo: 1000.0,        // LOAD THE CANNONS (High priority)
            gunner_working_bonus: 50.0,
            gunner_distance_factor: 500.0, // Extreme pull to cannons

            firefighter_base_reward: 5000.0, // Increased from 2000.0
            firefighter_distance_factor: 50.0, // Increased from 10.0

            healing_reward: 2000.0, // Increased to prioritize survival (was 500)
            sickbay_distance_factor: 50.0,

            backtracking_penalty: 200.0, // WAS 50.0. Prevent infinite loops (Slippery Deck).
            commitment_bonus: 500.0,     // Reward for consistency

            solution_solver_reward: 60000.0, // WAS 40000.0. Solving problems is key.
            solution_distance_factor: 100.0,
            situation_logistics_reward: 25000.0, // WAS 5000.0. GO GET THE ITEM.
            situation_resolved_reward: 30000.0,  // New: Big payout for finishing the job
            system_importance_multiplier: 2.0,   // Critical systems are 2x more important

            ammo_stockpile_reward: 5000.0, // Encourage dumping ammo in Cannons (Huge)
            loose_ammo_reward: 200.0,      // Every nut on the map is hope
            hazard_proximity_reward: 50.0,
            situation_exposure_penalty: 1000.0,
            system_disabled_penalty: 50000.0, // WAS 25000.0. Broken guns = Death.
            shooting_reward: 22000.0,         // Increased from 20000.0 to finish bosses

            scavenger_reward: 500.0,
            repair_proximity_reward: 1000.0,
            cargo_repair_incentive: 25.0,
            cargo_repair_proximity_reward: 1.0, // New

            boss_level_reward: 20000.0,
            turn_penalty: 12000.0, // Increased from 5000.0 to prevent stalling
            step_penalty: 100.0,   // WAS 5.0. Prevent free-action loops.

            checkmate_threshold: 20.0,
            checkmate_multiplier: 8.0,
            checkmate_max_mult: 100.0, // Cap the reward to prevent suicidal aggression

            // Critical State
            critical_hull_threshold: 16.0,
            critical_hull_penalty_base: 150_000.0,
            critical_hull_penalty_per_hp: 50_000.0,
            critical_fire_threshold: 2,
            critical_fire_penalty_per_token: 25_000.0,
            critical_system_hazard_penalty: 50_000.0, // Major penalty
            critical_survival_mult: 0.1,
            critical_threat_mult: 5.0,

            // Exponents
            hull_exponent: 2.2,
            fire_exponent: 2.0,
            cargo_repair_exponent: 1.5,
            hull_risk_exponent: 1.1,
            panic_fire_exponent: 2.0,  // Quadratic panic
            panic_hull_exponent: 1.5,  // Accelerated panic
            checkmate_exponent: 1.5,   // New
            hull_penalty_scaling: 1.1, // Reduced from 1.2 to reduce stalling

            // Multipliers
            fire_urgency_mult: 5.0,
            hazard_proximity_range: 20.0,
            gunner_dist_range: 20.0,
            gunner_per_ammo_mult: 0.2,
            gunner_en_route_mult: 0.1,
            gunner_wheelbarrow_penalty: 0.1,
            baker_wheelbarrow_mult: 2.0,

            threat_severe_reward: 2000.0,
            threat_mitigated_reward: 500.0,
            threat_hull_risk_mult: 0.5,
            threat_shield_waste_penalty: 100.0,
        }
    }
}
/// Calculates the minimum distance from start_room to any room in `targets`.
/// Returns 999 if unreachable.
fn min_distance(state: &GameState, start: RoomId, targets: &HashSet<RoomId>) -> u32 {
    if targets.contains(&start) {
        return 0;
    }

    let mut queue = VecDeque::new();
    queue.push_back((start, 0));
    let mut visited = HashSet::new();
    visited.insert(start);

    while let Some((current, dist)) = queue.pop_front() {
        if targets.contains(&current) {
            return dist;
        }

        if let Some(room) = state.map.rooms.get(&current) {
            for &neighbor in &room.neighbors {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    queue.push_back((neighbor, dist + 1));
                }
            }
        }
    }
    999
}

/// Calculates the total score for a search node, combining static state evaluation
/// and transition penalties/rewards (e.g., hull loss).
pub fn calculate_score(
    parent: &GameState,
    current: &GameState,
    history: &[&(PlayerId, GameAction)],
    weights: &BeamScoringWeights,
) -> f64 {
    let mut score = score_static(current, history, weights);
    score += score_transition(parent, current, weights);

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
            let old_dist = min_distance(parent, old_room, &target_rooms);
            let new_dist = min_distance(current, new_room, &target_rooms);

            if new_dist < old_dist {
                score += weights.commitment_bonus;
            }
        }
    }

    score
}

/// Calculates a score for a single state snapshot.
/// Higher is better.
pub fn score_static(
    state: &GameState,
    history: &[&(PlayerId, GameAction)],
    weights: &BeamScoringWeights,
) -> f64 {
    // Terminal States
    if state.phase == GamePhase::Victory {
        return 1_000_000.0 + (state.hull_integrity as f64 * 1000.0);
    }
    // SUICIDE PREVENTION: Death must be worse than any living hell.
    // Previous penalties reached -17M, so Death must be -1B.
    if state.phase == GamePhase::GameOver || state.hull_integrity <= 0 {
        return -1_000_000_000.0;
    }

    // PROJECTED DAMAGE: Fires hurt hull at end of round. Count them now.
    let fire_count_total: usize = state
        .map
        .rooms
        .values()
        .map(|r| r.hazards.iter().filter(|h| **h == HazardType::Fire).count())
        .sum();

    let projected_hull = state.hull_integrity - fire_count_total as i32;
    // If fires will kill us this round, it's over.
    if projected_hull <= 0 {
        return -1_000_000_000.0;
    }

    let mut score = 0.0;
    // Use PROJECTED hull for critical status
    let is_critical = (projected_hull as f64) <= weights.critical_hull_threshold;

    // Penalty scaling based on hull integrity: lower hull = higher penalties for everything else
    let hull_penalty_scaler =
        (MAX_HULL as f64 / (projected_hull as f64).max(1.0)).powf(weights.hull_penalty_scaling);

    let threat_mult = if is_critical {
        weights.critical_threat_mult
    } else {
        1.0
    } * hull_penalty_scaler;

    // --- Checkmate Heuristic ---
    let mut checkmate_mult = 1.0;
    if (state.enemy.hp as f64) <= weights.checkmate_threshold && state.enemy.hp > 0 {
        let progress = weights.checkmate_threshold / (state.enemy.hp as f64);
        checkmate_mult = (progress.powf(weights.checkmate_exponent) * weights.checkmate_multiplier)
            .min(weights.checkmate_max_mult);
    }

    // --- Critical State Heuristic ---
    // Override everything if we are about to die.
    // Survival is absolute. Do not reduce panic based on enemy health.

    if is_critical {
        let mut panic_score = -weights.critical_hull_penalty_base;
        let deficit = (weights.critical_hull_threshold + 1.0 - projected_hull as f64).max(0.0);
        panic_score -= deficit.powf(weights.panic_hull_exponent)
            * weights.critical_hull_penalty_per_hp
            * hull_penalty_scaler;

        score += panic_score;
    }

    if fire_count_total >= weights.critical_fire_threshold {
        // Massive penalty for uncontrolled fire
        let excess_fire =
            (fire_count_total as f64 - weights.critical_fire_threshold as f64 + 1.0).max(1.0);
        score -= excess_fire.powf(weights.panic_fire_exponent)
            * weights.critical_fire_penalty_per_token
            * hull_penalty_scaler;
    }

    // --- 1. Vital Stats & Progression ---
    // Non-linear Hull Penalty: Penalize missing hull exponentially (Square of missing health)
    // Use PROJECTED hull
    let missing_hull = (MAX_HULL as f64 - projected_hull as f64).max(0.0);
    score -= missing_hull.powf(weights.hull_exponent) * weights.hull_integrity;

    score += state.boss_level as f64 * weights.boss_level_reward;

    // Bloodlust: If enemy is low, increase reward for damage
    let enemy_hp_percent = if state.enemy.max_hp > 0 {
        state.enemy.hp as f64 / state.enemy.max_hp as f64
    } else {
        0.0
    };
    let bloodlust_mult = if enemy_hp_percent < 0.5 { 1.5 } else { 1.0 };

    score += (state.enemy.max_hp as f64 - state.enemy.hp as f64)
        * weights.enemy_hp
        * checkmate_mult
        * bloodlust_mult;

    score -= state.turn_count as f64 * weights.turn_penalty;
    score -= history.len() as f64 * weights.step_penalty;

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
    let missing_hull_percent =
        ((MAX_HULL as f64 - projected_hull as f64) / MAX_HULL as f64).max(0.0);
    let hull_risk_mult = 1.0 + (missing_hull_percent * 10.0).powf(2.5);

    score -= (fire_rooms.len() as f64).powf(weights.fire_exponent)
        * weights.fire_penalty_base
        * hull_risk_mult
        * threat_mult; // Apply threat multiplier (which includes hull_penalty_scaler)
    score -= water_count as f64 * weights.water_penalty;

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
                score -= weights.system_disabled_penalty
                    * weights.system_importance_multiplier
                    * hull_penalty_scaler;
            }

            // New: Penalty for ANY hazard in a critical room (Danger Zone)
            // Even if not disabled yet (e.g. fire just started), we want to clear it ASAP.
            score -= weights.critical_system_hazard_penalty
                * (room.hazards.len() as f64)
                * weights.system_importance_multiplier
                * hull_penalty_scaler;
        }
    }

    // -- Cargo Hull Repair Incentive --
    let urgency = (MAX_HULL as f64 - state.hull_integrity as f64)
        .max(0.0)
        .powf(weights.cargo_repair_exponent);

    // Reward for being in Cargo scales with damage
    let dynamic_incentive = urgency * weights.cargo_repair_incentive;

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
                score += dynamic_incentive;
            }
            // New: Proximity Reward (Move towards Cargo when damaged)
            if urgency > 0.0 {
                let dist = min_distance(state, p.room_id, &t_set);
                if dist != 999 {
                    score += (20.0 - dist as f64).max(0.0)
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

    score -= (negative_situations as f64).powf(1.5) * weights.active_situation_penalty;

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
                score -= count as f64 * weights.situation_exposure_penalty;
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
                score += weights.threat_severe_reward * hull_urgency_mult; // High value for mitigating a real threat
            } else {
                score += weights.threat_mitigated_reward; // Small value for safety
            }

            // Defensive Posture Heuristic: Reward safety when critical
            if state.hull_integrity < 15 {
                score += 10000.0;
            }
        } else {
            // Not protected - Apply penalties
            if players_in_target > 0 {
                score -= players_in_target as f64 * weights.threat_player_penalty * threat_mult;
            }

            if let Some(sys) = attack.target_system
                && matches!(
                    sys,
                    SystemType::Engine | SystemType::Cannons | SystemType::Bridge
                )
            {
                score -= weights.threat_system_penalty * threat_mult;
            }
            // General hull risk
            score -= weights.threat_system_penalty * weights.threat_hull_risk_mult * threat_mult;
        }
    } else {
        // No attack coming, but shields up? Wasted AP mostly, but small safety bonus
        if protected {
            score -= weights.threat_shield_waste_penalty;
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
            score -= weights.death_penalty;
            continue;
        }

        score += p.hp as f64 * weights.player_hp;
        score += p.ap as f64 * weights.ap_balance;

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
            _ => None,
        };

        // Dynamic Role Override: Critical Health (Seek Healing)
        if p.hp < 2 {
            assigned_room = None;
        }

        // Dynamic Role Override: Fire Panic
        // If there is ANY fire, only Gunners stick to their posts (unless fire is in Cannons).
        // Everyone else becomes a firefighter.
        if fire_count_total > 0 {
            if matches!(p.id.as_str(), "P5" | "P6") {
                // Gunners only leave if it's really bad or fire is IN the cannons
                if fire_count_total >= weights.critical_fire_threshold
                    || find_room_with_system(state, SystemType::Cannons)
                        .and_then(|id| state.map.rooms.get(&id))
                        .is_some_and(|r| r.hazards.contains(&HazardType::Fire))
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
            score += weights.station_keeping_reward;
        }
        // Removed distance penalty to prevent interference with temporary tasks

        // -- Role: Healer --
        if p.hp < 3 {
            let urgency = (3 - p.hp).pow(2) as f64;
            let dist = min_distance(state, p.room_id, &sickbay_rooms);
            if dist == 0 {
                score += weights.healing_reward * urgency;
            } else {
                score += (20.0 - dist as f64).max(0.0) * weights.sickbay_distance_factor * urgency;
            }
        }

        // -- Role: Gunner Logic --
        // P5/P6 are strictly Gunners via Station Keeping, but generic logic applies too.
        if peppernuts > 0 {
            let mut gunner_score = 0.0;
            let dist = min_distance(state, p.room_id, &cannon_rooms);

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
            score += gunner_score;
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
            let d = min_distance(state, p.room_id, &t_set);

            // Update Global Hazard Coverage (lowest distance wins)
            if let Some(min_d) = hazard_min_dists.get_mut(&hid)
                && d < *min_d
            {
                *min_d = d;
            }

            // --- Individual Role Logic ---
            let has_fire = room.hazards.contains(&HazardType::Fire);

            let is_critical = matches!(
                room.system,
                Some(SystemType::Engine) | Some(SystemType::Cannons) | Some(SystemType::Bridge)
            );

            if d < best_target_dist {
                best_target_dist = d;
                is_critical_target = is_critical;
                best_target_is_fire = has_fire;
            } else if d == best_target_dist {
                // Tie-breaker: Critical system wins
                if is_critical && !is_critical_target {
                    is_critical_target = true;
                    best_target_is_fire = has_fire;
                }
            }
        }

        if best_target_dist != 999 {
            // Panic Multiplier: If we are critical, firefighting is GOD.
            // If hull is projected to be low, we MUST extinguish.
            let panic_mult = if is_critical { 20.0 } else { 1.0 };

            if best_target_dist == 0 {
                emergency_score += weights.firefighter_base_reward * panic_mult;
            } else {
                emergency_score += (20.0 - best_target_dist as f64).max(0.0)
                    * weights.firefighter_distance_factor
                    * panic_mult;
            }

            // Critical Bonus (was "Targeted Repair")
            if is_critical_target {
                emergency_score +=
                    (20.0 - best_target_dist as f64).max(0.0) * weights.repair_proximity_reward;
            }

            // Extinguisher Bonus
            if best_target_is_fire && has_extinguisher {
                emergency_score *= 1.5;
            }

            score += emergency_score;
        }

        // -- Role: Scavenger (Ammo Gathering) --
        // Encourage gathering if low on ammo. Stronger with Wheelbarrow.
        if peppernuts < nut_cap && state.enemy.hp > 0 {
            let mut scavenger_score = 0.0;

            if !ammo_sources.is_empty() {
                let d = min_distance(state, p.room_id, &ammo_sources);
                scavenger_score += (20.0 - d as f64).max(0.0) * weights.scavenger_reward;
            }

            if has_wheelbarrow {
                scavenger_score *= weights.baker_wheelbarrow_mult;
            }
            score += scavenger_score;
        }
    }

    // --- Apply Global Hazard Coverage Reward ---
    for (_, min_d) in hazard_min_dists {
        if (min_d as f64) < weights.hazard_proximity_range {
            score += (weights.hazard_proximity_range - min_d as f64)
                * weights.hazard_proximity_reward
                * fire_urgency_mult;
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
                        let d = min_distance(state, p.room_id, &t_set);
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

                    if best_q_d == 0 && best_n_d == 0 {
                        score += weights.solution_solver_reward;
                    } else {
                        // Pull the qualified person
                        score +=
                            (20.0 - best_q_d as f64).max(0.0) * weights.solution_distance_factor;
                        // Pull the N-th person
                        if req_n > 1 && best_n_d != 999 {
                            score += (20.0 - best_n_d as f64).max(0.0)
                                * weights.solution_distance_factor;
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

                    if !sources.is_empty() {
                        let mut min_source_dist = 999;
                        for p in state.players.values() {
                            if p.hp > 0 {
                                let d = min_distance(state, p.room_id, &sources);
                                if d < min_source_dist {
                                    min_source_dist = d;
                                }
                            }
                        }
                        if min_source_dist != 999 {
                            score += (20.0 - min_source_dist as f64).max(0.0)
                                * weights.situation_logistics_reward;
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
        score += nuts as f64 * weights.loose_ammo_reward;

        if room.system == Some(SystemType::Cannons) {
            // Massive reward for having ammo at Cannons
            score += nuts as f64 * weights.ammo_stockpile_reward;
        }
    }

    // --- 6. Trajectory Heuristics (Anti-Oscillation) ---
    // (Kept same as before)
    let mut player_moves: HashMap<PlayerId, Vec<(usize, u32)>> = HashMap::new();
    for (idx, item) in history.iter().enumerate() {
        let (pid, act) = *item;
        if matches!(act, GameAction::Shoot) {
            score += weights.shooting_reward;
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
                    score -= weights.backtracking_penalty;
                }
            }
        }
    }

    score
}

/// Calculates the score delta based on the transition from parent to current state.
fn score_transition(parent: &GameState, current: &GameState, weights: &BeamScoringWeights) -> f64 {
    let mut score = 0.0;
    // Hull Delta Penalty: Penalize the ACT of losing hull
    let hull_loss = (parent.hull_integrity as f64 - current.hull_integrity as f64).max(0.0);
    score -= hull_loss * weights.hull_delta_penalty;

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
        score += weights.situation_resolved_reward;
    }

    score
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
