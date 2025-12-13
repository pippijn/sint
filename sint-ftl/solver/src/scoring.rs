use sint_core::types::{
    GameAction, GamePhase, GameState, HazardType, ItemType, PlayerId, RoomId, SystemType, MAX_HULL,
};
use std::collections::{HashMap, HashSet, VecDeque};

/// Hyperparameters for the scoring function.
#[derive(Debug, Clone)]
pub struct ScoringWeights {
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

    // Situation Solving
    pub solution_solver_reward: f64,
    pub solution_distance_factor: f64,
    pub situation_logistics_reward: f64, // New: Reward for getting items needed for situations

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

    // Progression
    pub boss_level_reward: f64,
    pub turn_penalty: f64,
    pub step_penalty: f64,

    // Checkmate
    pub checkmate_threshold: f64,
    pub checkmate_multiplier: f64,

    // Critical State / Panic
    pub critical_hull_threshold: f64,
    pub critical_hull_penalty_base: f64,
    pub critical_hull_penalty_per_hp: f64,
    pub critical_fire_threshold: usize,
    pub critical_fire_penalty_per_token: f64,

    // Exponents (Non-Linearity)
    pub hull_exponent: f64,
    pub fire_exponent: f64,
    pub cargo_repair_exponent: f64,
    pub hull_risk_exponent: f64,

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

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            hull_integrity: 8000.0, // Critical: Hull is Life (Increased to prioritize survival)
            hull_delta_penalty: 20000.0, // Immediate penalty for taking damage in a step
            enemy_hp: 15000.0,      // High priority: KILL THE BOSS
            player_hp: 200.0,
            ap_balance: 50.0, // High value on AP = Don't waste turns

            // Hazards - significantly increased penalties
            fire_penalty_base: 5000.0, // Burn, baby, burn (but don't)
            water_penalty: 1000.0,

            // Situations & Threats
            active_situation_penalty: 50000.0, // WAS 8000.0 - HUGE increase. Situations kill runs.
            threat_player_penalty: 100.0,      // Reduced to allow calculated risks
            threat_system_penalty: 2000.0,
            death_penalty: 50000.0,

            // Roles
            station_keeping_reward: 3000.0, // Strict discipline for efficiency
            gunner_base_reward: 600.0,      // STAY AT THE CANNONS
            gunner_per_ammo: 1000.0,        // LOAD THE CANNONS (High priority)
            gunner_working_bonus: 50.0,
            gunner_distance_factor: 500.0, // Extreme pull to cannons

            firefighter_base_reward: 2000.0,
            firefighter_distance_factor: 10.0,

            healing_reward: 500.0, // Increased to prioritize survival (was 50)
            sickbay_distance_factor: 50.0,

            backtracking_penalty: 200.0, // WAS 50.0. Prevent infinite loops (Slippery Deck).

            solution_solver_reward: 60000.0, // WAS 20000.0. Solving problems is as important as avoiding damage.
            solution_distance_factor: 100.0,
            situation_logistics_reward: 5000.0,

            ammo_stockpile_reward: 5000.0, // Encourage dumping ammo in Cannons (Huge)
            loose_ammo_reward: 200.0,      // Every nut on the map is hope
            hazard_proximity_reward: 50.0,
            situation_exposure_penalty: 1000.0,
            system_disabled_penalty: 5000.0,
            shooting_reward: 250.0, // SHOOT! (Reduced to favor actual damage)

            scavenger_reward: 500.0,
            repair_proximity_reward: 1000.0,
            cargo_repair_incentive: 25.0,

            boss_level_reward: 20000.0,
            turn_penalty: 500.0, // Increased to prevent stalling
            step_penalty: 20.0,  // WAS 5.0. Prevent free-action loops.

            checkmate_threshold: 15.0,
            checkmate_multiplier: 2.5,

            // Critical State
            critical_hull_threshold: 5.0,
            critical_hull_penalty_base: 100_000.0,
            critical_hull_penalty_per_hp: 50_000.0,
            critical_fire_threshold: 3,
            critical_fire_penalty_per_token: 25_000.0,

            // Exponents
            hull_exponent: 1.5,
            fire_exponent: 3.0,
            cargo_repair_exponent: 1.5,
            hull_risk_exponent: 1.1,

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
    weights: &ScoringWeights,
) -> f64 {
    let mut score = score_static(current, history, weights);
    score += score_transition(parent, current, weights);
    score
}

/// Calculates a score for a single state snapshot.
/// Higher is better.
fn score_static(
    state: &GameState,
    history: &[&(PlayerId, GameAction)],
    weights: &ScoringWeights,
) -> f64 {
    // Terminal States
    if state.phase == GamePhase::Victory {
        return 1_000_000.0 + (state.hull_integrity as f64 * 1000.0);
    }
    if state.phase == GamePhase::GameOver || state.hull_integrity <= 0 {
        return -1_000_000.0;
    }

    let mut score = 0.0;

    // --- Checkmate Heuristic ---
    let mut checkmate_mult = 1.0;
    if (state.enemy.hp as f64) <= weights.checkmate_threshold && state.enemy.hp > 0 {
        checkmate_mult = weights.checkmate_multiplier;
    }

    // --- Critical State Heuristic ---
    // Override everything if we are about to die, UNLESS we are winning (Checkmate).
    // Scale panic by enemy health: If enemy is near death, panic less.
    let enemy_health_factor = (state.enemy.hp as f64 / 20.0).min(1.0);

    if (state.hull_integrity as f64) <= weights.critical_hull_threshold {
        let mut panic_score = -weights.critical_hull_penalty_base;
        panic_score -= (weights.critical_hull_threshold + 1.0 - state.hull_integrity as f64)
            * weights.critical_hull_penalty_per_hp;

        // Apply factor: If enemy is low, reduce panic impact to allow risk
        score += panic_score * enemy_health_factor;
    }

    // Count fires for critical check
    let fire_count_total: usize = state
        .map
        .rooms
        .values()
        .map(|r| r.hazards.iter().filter(|h| **h == HazardType::Fire).count())
        .sum();

    if fire_count_total >= weights.critical_fire_threshold {
        // Massive penalty for uncontrolled fire
        score -= (fire_count_total as f64) * weights.critical_fire_penalty_per_token;
    }

    // --- 1. Vital Stats & Progression ---
    // Non-linear Hull Penalty: Penalize missing hull exponentially (Square of missing health)
    let missing_hull = (MAX_HULL as f64 - state.hull_integrity as f64).max(0.0);
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
    let hull_risk_factor = 1.0
        + (MAX_HULL as f64 - state.hull_integrity as f64)
            .max(0.0)
            .powf(weights.hull_risk_exponent);
    score -= (fire_count as f64).powf(weights.fire_exponent)
        * weights.fire_penalty_base
        * hull_risk_factor;
    score -= water_count as f64 * weights.water_penalty;

    // -- System Disabled --
    for room in state.map.rooms.values() {
        if let Some(sys) = room.system {
            if matches!(
                sys,
                SystemType::Engine | SystemType::Cannons | SystemType::Bridge
            ) {
                if !room.hazards.is_empty() {
                    let has_disabling_hazard = room.hazards.contains(&HazardType::Fire)
                        || room.hazards.contains(&HazardType::Water);
                    if has_disabling_hazard {
                        score -= weights.system_disabled_penalty;
                    }
                }
            }
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
        for p in state.players.values() {
            if p.room_id == cid {
                score += dynamic_incentive;
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
    score -= (state.active_situations.len() as f64).powf(1.5) * weights.active_situation_penalty;

    for card in &state.active_situations {
        if card.id == sint_core::types::CardId::Overheating {
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
            if is_severe {
                score += weights.threat_severe_reward; // High value for mitigating a real threat
            } else {
                score += weights.threat_mitigated_reward; // Small value for safety
            }
        } else {
            // Not protected - Apply penalties
            if players_in_target > 0 {
                score -= players_in_target as f64 * weights.threat_player_penalty;
            }

            if let Some(sys) = attack.target_system {
                if matches!(
                    sys,
                    SystemType::Engine | SystemType::Cannons | SystemType::Bridge
                ) {
                    score -= weights.threat_system_penalty;
                }
            }
            // General hull risk
            score -= weights.threat_system_penalty * weights.threat_hull_risk_mult;
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
        // P1: Kitchen (Room 5)
        // P3: Bridge (Room 7)
        // P4: Engine (Room 4)
        // P5, P6: Cannons (Room 6)
        // P2: Hallway/Roaming (Room 0) - No strict station

        let mut assigned_room = match p.id.as_str() {
            "P1" => Some(5),
            "P3" => Some(7),
            "P4" => Some(4),
            "P5" | "P6" => Some(6),
            _ => None,
        };

        // Dynamic Role Override: Firefighter roams free
        if has_extinguisher {
            assigned_room = None;
        }
        // Dynamic Role Override: Critical Health (Seek Healing)
        if p.hp < 2 {
            assigned_room = None;
        }

        if let Some(target) = assigned_room {
            if p.room_id == target {
                score += weights.station_keeping_reward;
            }
            // Removed distance penalty to prevent interference with temporary tasks
        }

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
                    if let Some(r) = state.map.rooms.get(&rid) {
                        if !r.hazards.contains(&HazardType::Fire) {
                            gunner_score += weights.gunner_working_bonus;
                        }
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
        // If holding extinguisher, prioritize fire.
        let mut emergency_score = 0.0;
        let mut best_target_dist = 999;
        let mut is_critical_target = false;

        // Iterate all hazardous rooms to find best emergency target for THIS player
        // AND update global coverage stats.
        for &hid in &hazardous_rooms {
            let room = state.map.rooms.get(&hid).unwrap();

            // Calculate distance to this hazard for THIS player
            let mut t_set = HashSet::new();
            t_set.insert(hid);
            let d = min_distance(state, p.room_id, &t_set);

            // Update Global Hazard Coverage (lowest distance wins)
            if let Some(min_d) = hazard_min_dists.get_mut(&hid) {
                if d < *min_d {
                    *min_d = d;
                }
            }

            // --- Individual Role Logic ---
            let has_fire = room.hazards.contains(&HazardType::Fire);

            // If fire exists, only Extinguisher holder gets full priority (others can help but less efficient)
            if has_fire && !has_extinguisher {
                continue;
            }

            let is_critical = matches!(
                room.system,
                Some(SystemType::Engine) | Some(SystemType::Cannons) | Some(SystemType::Bridge)
            );

            if d < best_target_dist {
                best_target_dist = d;
                is_critical_target = is_critical;
            } else if d == best_target_dist && is_critical {
                // Tie-breaker: Critical system wins
                is_critical_target = true;
            }
        }

        if best_target_dist != 999 {
            if best_target_dist == 0 {
                emergency_score += weights.firefighter_base_reward;
            } else {
                emergency_score +=
                    (20.0 - best_target_dist as f64).max(0.0) * weights.firefighter_distance_factor;
            }

            // Critical Bonus (was "Targeted Repair")
            if is_critical_target {
                emergency_score +=
                    (20.0 - best_target_dist as f64).max(0.0) * weights.repair_proximity_reward;
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
        if let Some(sol) = &card.solution {
            if let Some(sys) = sol.target_system {
                let mut target_room = None;
                for r in state.map.rooms.values() {
                    if r.system == Some(sys) {
                        target_room = Some(r.id);
                        break;
                    }
                }
                if let Some(tid) = target_room {
                    let mut min_d = 999;
                    let mut someone_has_item = false;
                    let required_item = sol.item_cost.as_ref();

                    for p in state.players.values() {
                        if p.hp > 0 {
                            // Check Item Requirement
                            let meets_req = match required_item {
                                Some(item) => p.inventory.contains(item),
                                None => true,
                            };

                            if meets_req {
                                someone_has_item = true;
                                let mut t_set = HashSet::new();
                                t_set.insert(tid);
                                let d = min_distance(state, p.room_id, &t_set);
                                if d < min_d {
                                    min_d = d;
                                }
                            }
                        }
                    }
                    if min_d == 0 {
                        score += weights.solution_solver_reward;
                    } else if min_d != 999 {
                        score += (20.0 - min_d as f64).max(0.0) * weights.solution_distance_factor;
                    }

                    // Logistics: If needed item is missing, reward getting it
                    if !someone_has_item {
                        if let Some(item) = required_item {
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
fn score_transition(parent: &GameState, current: &GameState, weights: &ScoringWeights) -> f64 {
    let mut score = 0.0;
    // Hull Delta Penalty: Penalize the ACT of losing hull
    let hull_loss = (parent.hull_integrity as f64 - current.hull_integrity as f64).max(0.0);
    score -= hull_loss * weights.hull_delta_penalty;
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
