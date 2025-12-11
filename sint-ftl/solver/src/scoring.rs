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

    // Progression
    pub boss_level_reward: f64,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            hull_integrity: 3078.56,
            enemy_hp: 2000.0,
            player_hp: 100.0,
            ap_balance: 0.1,
            fire_penalty_base: 100.0,
            water_penalty: 50.0,
            active_situation_penalty: 176.01,
            threat_player_penalty: 300.0,
            threat_system_penalty: 300.0,
            death_penalty: 50000.0,
            gunner_base_reward: 50.0,
            gunner_per_ammo: 300.0,
            gunner_working_bonus: 20.0,
            gunner_distance_factor: 10.0,
            firefighter_base_reward: 83.0,
            firefighter_distance_factor: 10.0,
            baker_base_reward: 20.0,
            baker_distance_factor: 5.0,
            healing_reward: 81.23,
            sickbay_distance_factor: 10.0,
            backtracking_penalty: 49.7,
            solution_solver_reward: 100.0,
            solution_distance_factor: 10.0,
            ammo_stockpile_reward: 50.0,
            boss_level_reward: 10000.0,
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
    // Hull: Base survival. Weight high.
    score += state.hull_integrity as f64 * weights.hull_integrity;

    // Boss Level: Major milestone.
    score += state.boss_level as f64 * weights.boss_level_reward;

    // Enemy HP: The Goal. Weight high.
    // We use (Max - Current) so we maximize damage dealt.
    score += (state.enemy.max_hp - state.enemy.hp) as f64 * weights.enemy_hp;

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

    // Fire grows and damages systems/players. Exponential penalty.
    score -= (fire_count as f64).powf(1.5) * weights.fire_penalty_base;

    // Water restricts movement/systems. Linear penalty.
    score -= water_count as f64 * weights.water_penalty;

    // --- 3. Active Situations ---
    // Each active situation is a problem.
    score -= state.active_situations.len() as f64 * weights.active_situation_penalty;

    // --- 4. Pending Threats (Telegraphs) ---
    // If the enemy is about to attack, and we are not protected, that's bad.
    let protected = state.shields_active || state.evasion_active;

    if let Some(attack) = &state.enemy.next_attack {
        if !protected {
            // Targeting a room with players?
            let players_in_target = state
                .players
                .values()
                .filter(|p| p.room_id == attack.target_room)
                .count();
            if players_in_target > 0 {
                // Danger!
                score -= players_in_target as f64 * weights.threat_player_penalty;
            }

            // Targeting a critical system?
            if let Some(sys) = attack.target_system {
                // Losing Engines/Cannons is bad
                if matches!(
                    sys,
                    SystemType::Engine | SystemType::Cannons | SystemType::Bridge
                ) {
                    score -= weights.threat_system_penalty;
                }
            }
        }
    }

    // --- 5. Player Heuristics ---
    let mut cannon_rooms = HashSet::new();
    let mut kitchen_rooms = HashSet::new();
    let mut sickbay_rooms = HashSet::new();
    let mut fire_rooms = HashSet::new();
    let mut nut_locations = HashSet::new();

    // Scan rooms for systems and hazards
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
        if room.hazards.contains(&HazardType::Fire) {
            fire_rooms.insert(room.id);
        }
        if room.items.contains(&ItemType::Peppernut) {
            nut_locations.insert(room.id);
        }
    }

    for p in state.players.values() {
        if p.hp <= 0 {
            score -= weights.death_penalty; // Avoid death
            continue;
        }

        // Reward for Player HP (Survival)
        score += p.hp as f64 * weights.player_hp;

        // Action Points: Slight bonus for having AP
        score += p.ap as f64 * weights.ap_balance;

        // Inventory Analysis
        let peppernuts = p
            .inventory
            .iter()
            .filter(|i| **i == ItemType::Peppernut)
            .count();
        let has_wheelbarrow = p.inventory.contains(&ItemType::Wheelbarrow);
        let has_extinguisher = p.inventory.contains(&ItemType::Extinguisher);
        let nut_cap = if has_wheelbarrow { 5 } else { 1 };

        // -- Role: Healer (Needs Health) --
        if p.hp < 3 {
            let urgency = (3 - p.hp).pow(2) as f64; // 1->4, 2->1
            let dist = min_distance(state, p.room_id, &sickbay_rooms);
            if dist == 0 {
                score += weights.healing_reward * urgency;
            } else {
                score += (20.0 - dist as f64).max(0.0) * weights.sickbay_distance_factor * urgency;
            }
        }

        // -- Role: Gunner (Has Peppernut) --
        if peppernuts > 0 {
            let mut gunner_score = 0.0;
            let dist = min_distance(state, p.room_id, &cannon_rooms);
            if dist == 0 {
                // In Cannons with Ammo! Very Good.
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
                // Move closer
                gunner_score += (20.0 - dist as f64).max(0.0)
                    * weights.gunner_distance_factor
                    * peppernuts as f64;
            }

            // Penalty for partial loads with Wheelbarrow (Efficiency)
            if has_wheelbarrow && peppernuts < 5 {
                gunner_score *= 0.1;
            }

            score += gunner_score;
        }

        // -- Role: Firefighter (Has Extinguisher) --
        if has_extinguisher {
            if !fire_rooms.is_empty() {
                let dist = min_distance(state, p.room_id, &fire_rooms);
                if dist == 0 {
                    // In a room with fire! Good (can extinguish).
                    score += weights.firefighter_base_reward;
                } else {
                    // Move to fire
                    score += (20.0 - dist as f64).max(0.0) * weights.firefighter_distance_factor;
                }
            }
            // No penalty for holding it when no fire exists. Safety first!
        }

        // -- Role: Baker (Gather Ammo) --
        // If we have space for ammo
        if peppernuts < nut_cap && state.enemy.hp > 0 {
            let mut baker_score = 0.0;

            // Prefer existing nuts on floor (faster)
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

            // Bonus for Wheelbarrow gathering (Efficiency)
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
                    // Find closest player (exclude fainted)
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
        if room.system == Some(SystemType::Cannons) {
            let nuts = room
                .items
                .iter()
                .filter(|i| **i == ItemType::Peppernut)
                .count();
            score += nuts as f64 * weights.ammo_stockpile_reward;
        }
    }

    // --- 6. Trajectory Heuristics (Anti-Oscillation) ---
    // Detect Move(A) -> ... -> Move(B) -> ... -> Move(A)
    // where no "useful" action occurred in between.

    // 1. Build per-player timeline of moves and relevant events
    // Map: PlayerId -> List of (HistoryIndex, RoomId) for moves
    let mut player_moves: HashMap<PlayerId, Vec<(usize, u32)>> = HashMap::new();

    // 2. Scan history to populate moves and identify interaction timestamps
    for (idx, (pid, act)) in history.iter().enumerate() {
        if let GameAction::Move { to_room } = act {
            player_moves
                .entry(pid.clone())
                .or_default()
                .push((idx, *to_room));
        }
    }

    // 3. Check for backtracks
    for (pid, moves) in player_moves {
        if moves.len() < 3 {
            continue;
        }

        // We look for patterns in the sequence of *destinations*.
        // If we have moves to: A, B, A
        // The indices in history are idx1, idx2, idx3.
        // The interval to check is (idx2 + 1) .. idx3.
        // Wait: The player is AT 'A' after idx1. They move to 'B' at idx2. They move back to 'A' at idx3.
        // The "useful" action must happen while they are at 'B'.
        // That corresponds to the interval [idx2 + 1, idx3 - 1].

        for i in 0..moves.len() - 2 {
            let (_idx_a_start, room_a) = moves[i];
            let (idx_b, _room_b) = moves[i + 1];
            let (idx_a_return, room_a_return) = moves[i + 2];

            if room_a == room_a_return {
                // Backtrack detected: A -> B -> A
                // Check for useful actions in history interval (idx_b, idx_a_return)
                // This interval represents the time spent at Room B.

                let mut useful = false;
                for j in (idx_b + 1)..idx_a_return {
                    if let Some((actor, action)) = history.get(j) {
                        // Case A: Player did something useful themselves
                        if actor == &pid {
                            match action {
                                GameAction::Move { .. } => {} // Should not happen in this logic as we iterate moves
                                GameAction::Pass
                                | GameAction::VoteReady { .. }
                                | GameAction::Undo { .. }
                                | GameAction::Chat { .. } => {}
                                _ => {
                                    useful = true;
                                    break;
                                } // Interact, Shoot, Extinguish, etc.
                            }
                        }

                        // Case B: Player was targeted by someone else
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
/// Tracks the "Area Under the Curve" for Hull and Hazards.
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

    /// Update the accumulator with a state at the end of a round
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

        // 1. Victory Bonus (Massive)
        if self.victory {
            score += 1_000_000.0;
        }

        // 2. Hull Integral (The "Living Well" metric)
        // Living 10 rounds with 20 HP = 200 pts.
        // Living 10 rounds with 10 HP = 100 pts.
        // Weight: 10.
        score += self.total_hull_integral * 10.0;

        // 3. Hazard Integral (The "Chaos" metric)
        // Weight: -5.
        score -= self.total_hazard_integral * 5.0;

        // 4. Enemy HP Integral (The "Dominance" metric)
        // Rewards dealing damage EARLY.
        // Weight: -20 (Must be > SurvivalBonus / MinBossHP to discourage farming).
        score -= self.total_enemy_hp_integral * 20.0;

        // 5. Round Bonus (If not victory, strictly better to live longer)
        // If victory, faster is better?
        if self.victory {
            // Faster victory bonus: Penalty for rounds taken
            score -= self.rounds_survived as f64 * 100.0;
        } else {
            // Survival bonus
            score += self.rounds_survived as f64 * 100.0;
        }

        score
    }
}
