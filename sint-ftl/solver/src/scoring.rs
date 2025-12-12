use sint_core::types::{
    GameAction, GamePhase, GameState, HazardType, ItemType, PlayerId, RoomId, SystemType,
};
use std::collections::{HashMap, HashSet, VecDeque};

/// Hyperparameters for the scoring function.
#[derive(Debug, Clone)]
pub struct ScoringWeights {
    // Vital Stats
    pub hull_integrity: f64,
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

    pub baker_base_reward: f64,
    pub baker_distance_factor: f64,

    pub healing_reward: f64,
    pub sickbay_distance_factor: f64,

    // Anti-Oscillation
    pub backtracking_penalty: f64,

    // Situation Solving
    pub solution_solver_reward: f64,
    pub solution_distance_factor: f64,

    // Logistics
    pub ammo_stockpile_reward: f64,
    pub loose_ammo_reward: f64,
    pub hazard_proximity_reward: f64,
    pub situation_exposure_penalty: f64,
    pub system_disabled_penalty: f64,

    // Progression
    pub boss_level_reward: f64,
    pub turn_penalty: f64,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            hull_integrity: 2000.0, // Critical: Hull is Life (Decreased to allow trading blows)
            enemy_hp: 3000.0,        // High value on Damage (Increased to prioritize offense)
            player_hp: 100.0,
            ap_balance: 0.1,

            // Hazards - significantly increased penalties
            fire_penalty_base: 10000.0, // Burn, baby, burn (but don't)
            water_penalty: 1000.0,

            // Situations & Threats
            active_situation_penalty: 500.0,
            threat_player_penalty: 100.0, // Reduced to allow calculated risks
                                    threat_system_penalty: 2000.0,
                                    death_penalty: 50000.0,
                        
                                    // Roles
                                    station_keeping_reward: 5000.0, // Critical: Must be in position to react
                                    gunner_base_reward: 600.0, // STAY AT THE CANNONS
                                    gunner_per_ammo: 300.0, // LOAD THE CANNONS
                                    gunner_working_bonus: 50.0,
                                    gunner_distance_factor: 500.0, // Extreme pull to cannons
                        
                                    firefighter_base_reward: 2000.0,
                                    firefighter_distance_factor: 10.0,
                        
                                    baker_base_reward: 300.0, // CRITICAL: We need ammo
                                    baker_distance_factor: 20.0,
                        
                                    healing_reward: 50.0,
                                    sickbay_distance_factor: 10.0,
                        
                                    backtracking_penalty: 50.0,
                        
                                    solution_solver_reward: 200.0,
                                    solution_distance_factor: 10.0,
                        
                                    ammo_stockpile_reward: 500.0, // Encourage dumping ammo in Cannons
                                    loose_ammo_reward: 200.0, // Every nut on the map is hope
                                    hazard_proximity_reward: 50.0,
                                    situation_exposure_penalty: 1000.0,
                                    system_disabled_penalty: 5000.0,
            boss_level_reward: 10000.0,
            turn_penalty: 50.0,
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

/// Calculates a score for a single state snapshot.
/// Higher is better.
pub fn score_state(
    state: &GameState,
    history: &[(PlayerId, GameAction)],
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

    // --- 1. Vital Stats & Progression ---
    score += state.hull_integrity as f64 * weights.hull_integrity;
    score += state.boss_level as f64 * weights.boss_level_reward;
    score += (state.enemy.max_hp - state.enemy.hp) as f64 * weights.enemy_hp;
    score -= state.turn_count as f64 * weights.turn_penalty;

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

    score -= (fire_count as f64).powf(1.3) * weights.fire_penalty_base;
    score -= water_count as f64 * weights.water_penalty;

    // -- System Disabled --
    for room in state.map.rooms.values() {
        if let Some(sys) = room.system {
            if matches!(sys, SystemType::Engine | SystemType::Cannons | SystemType::Bridge) {
                if !room.hazards.is_empty() {
                    let has_disabling_hazard = room.hazards.contains(&HazardType::Fire) || room.hazards.contains(&HazardType::Water);
                    if has_disabling_hazard {
                        score -= weights.system_disabled_penalty;
                    }
                }
            }
        }
    }

    // -- Hazard Proximity --
    let mut hazardous_rooms = fire_rooms.clone();
    for room in state.map.rooms.values() {
        if room.hazards.contains(&HazardType::Water) {
            hazardous_rooms.insert(room.id);
        }
    }

    for &hid in &hazardous_rooms {
        let mut min_d = 999;
        for p in state.players.values() {
            if p.hp > 0 {
                let mut t_set = HashSet::new();
                t_set.insert(hid);
                let d = min_distance(state, p.room_id, &t_set);
                if d < min_d {
                    min_d = d;
                }
            }
        }
        if min_d < 20 {
             score += (20.0 - min_d as f64) * weights.hazard_proximity_reward;
        }
    }

    // --- 3. Active Situations ---
    score -= state.active_situations.len() as f64 * weights.active_situation_penalty;

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
                let count = state
                    .players
                    .values()
                    .filter(|p| p.room_id == eid)
                    .count();
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
                score += 2000.0; // High value for mitigating a real threat
            } else {
                score += 500.0; // Small value for safety
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
            score -= weights.threat_system_penalty * 0.5;
        }
    } else {
        // No attack coming, but shields up? Wasted AP mostly, but small safety bonus
        if protected {
            score -= 100.0; 
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
        let assigned_room = match p.id.as_str() {
            "P1" => Some(5),
            "P3" => Some(7),
            "P4" => Some(4),
            "P5" | "P6" => Some(6),
            _ => None,
        };

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
            gunner_score += (20.0 - dist as f64).max(0.0) * weights.gunner_distance_factor * (1.0 + 0.2 * peppernuts as f64);

            if dist == 0 {
                // At cannons: Reward quantity heavily to encourage stocking/shooting
                gunner_score += weights.gunner_per_ammo * peppernuts as f64 + weights.gunner_base_reward;
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
                gunner_score += weights.gunner_per_ammo * 0.1 * peppernuts as f64;
            }
            
            if has_wheelbarrow && peppernuts < 5 {
                gunner_score *= 0.1;
            }
            score += gunner_score;
        }

        // -- Role: Firefighter --
        if has_extinguisher {
            if !fire_rooms.is_empty() {
                let dist = min_distance(state, p.room_id, &fire_rooms);
                if dist == 0 {
                    score += weights.firefighter_base_reward;
                } else {
                    score += (20.0 - dist as f64).max(0.0) * weights.firefighter_distance_factor;
                }
            }
        }

        // -- Role: Baker --
        // P1 is Baker via Station Keeping.
        // Encourage gathering if low on ammo
        if peppernuts < nut_cap && state.enemy.hp > 0 {
            let mut baker_score = 0.0;
            let dist_floor = if !nut_locations.is_empty() {
                min_distance(state, p.room_id, &nut_locations)
            } else {
                999
            };
            let dist_kitchen = min_distance(state, p.room_id, &kitchen_rooms);
            let target_dist = dist_floor.min(dist_kitchen);

            if target_dist == 0 {
                baker_score += weights.baker_base_reward;
            } else {
                baker_score += (20.0 - target_dist as f64).max(0.0) * weights.baker_distance_factor;
            }
            if has_wheelbarrow {
                baker_score *= 2.0;
            }
            score += baker_score;
        }
    }

    // --- 7. Situation Solving ---
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
                    for p in state.players.values() {
                        if p.hp > 0 {
                            let mut t_set = HashSet::new();
                            t_set.insert(tid);
                            let d = min_distance(state, p.room_id, &t_set);
                            if d < min_d {
                                min_d = d;
                            }
                        }
                    }
                    if min_d == 0 {
                        score += weights.solution_solver_reward;
                    } else {
                        score += (20.0 - min_d as f64).max(0.0) * weights.solution_distance_factor;
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
    for (idx, (pid, act)) in history.iter().enumerate() {
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
                    if let Some((actor, action)) = history.get(j) {
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
