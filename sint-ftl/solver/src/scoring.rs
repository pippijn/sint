use sint_core::types::{GamePhase, GameState, HazardType, ItemType, RoomId, SystemType};
use std::collections::{HashSet, VecDeque};

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
pub fn score_state(state: &GameState) -> f64 {
    // Terminal States
    if state.phase == GamePhase::Victory {
        return 1_000_000.0 + (state.hull_integrity as f64 * 1000.0);
    }
    if state.phase == GamePhase::GameOver || state.hull_integrity <= 0 {
        return -1_000_000.0;
    }

    let mut score = 0.0;

    // --- 1. Vital Stats ---
    // Hull: Base survival. Weight high.
    score += state.hull_integrity as f64 * 500.0;
    
    // Enemy HP: The Goal. Weight high.
    // We use (Max - Current) so we maximize damage dealt.
    score += (state.enemy.max_hp - state.enemy.hp) as f64 * 1000.0;

    // --- 2. Hazards ---
    let mut fire_rooms = HashSet::new();
    let mut water_count = 0;
    let mut fire_count = 0;

    for room in state.map.rooms.values() {
        let has_fire = room.hazards.contains(&HazardType::Fire);
        if has_fire {
            fire_rooms.insert(room.id);
            fire_count += room.hazards.iter().filter(|h| **h == HazardType::Fire).count();
        }
        water_count += room.hazards.iter().filter(|h| **h == HazardType::Water).count();
    }

    // Fire grows and damages systems/players. Exponential penalty.
    // 1 Fire = -50, 2 Fire = -141, 3 Fire = -260...
    score -= (fire_count as f64).powf(1.5) * 50.0;
    
    // Water restricts movement/systems. Linear penalty.
    score -= water_count as f64 * 20.0;

    // --- 3. Active Situations ---
    // Each active situation is a problem.
    score -= state.active_situations.len() as f64 * 100.0;

    // --- 4. Pending Threats (Telegraphs) ---
    // If the enemy is about to attack, and we are not protected, that's bad.
    let protected = state.shields_active || state.evasion_active;
    
    if let Some(attack) = &state.enemy.next_attack {
        if !protected {
            // Targeting a room with players?
            let players_in_target = state.players.values().filter(|p| p.room_id == attack.target_room).count();
            if players_in_target > 0 {
                // Danger!
                score -= players_in_target as f64 * 300.0;
            }

            // Targeting a critical system?
            if let Some(sys) = attack.target_system {
                // Losing Engines/Cannons is bad
                if matches!(sys, SystemType::Engine | SystemType::Cannons | SystemType::Bridge) {
                    score -= 100.0;
                }
            }
        }
    }

    // --- 5. Player Heuristics ---
    let cannon_room_set: HashSet<u32> = HashSet::from([7]); // Cannons
    let kitchen_room_set: HashSet<u32> = HashSet::from([6]); // Kitchen
    
    for p in state.players.values() {
        if p.hp <= 0 {
             score -= 1000.0; // Avoid death
             continue;
        }

        // Action Points: Slight bonus for having AP (optional, but finding efficient paths usually maximizes AP naturally)
        // Actually, we want to SPEND AP to do good things. But remaining AP implies flexibility.
        score += p.ap as f64 * 1.0;

        // -- Role: Gunner (Has Peppernut) --
        let peppernuts = p.inventory.iter().filter(|i| **i == ItemType::Peppernut).count();
        if peppernuts > 0 {
            let dist = min_distance(state, p.room_id, &cannon_room_set);
            if dist == 0 {
                // In Cannons with Ammo! Very Good.
                score += 50.0 * peppernuts as f64;
                // Bonus if Cannons system is working
                if let Some(r) = state.map.rooms.get(&7) {
                    if !r.hazards.contains(&HazardType::Fire) {
                         score += 20.0;
                    }
                }
            } else {
                // Move closer
                score += (20.0 - dist as f64).max(0.0) * 2.0 * peppernuts as f64;
            }
        }

        // -- Role: Firefighter (Has Extinguisher) --
        let has_extinguisher = p.inventory.iter().any(|i| *i == ItemType::Extinguisher);
        if has_extinguisher && !fire_rooms.is_empty() {
            let dist = min_distance(state, p.room_id, &fire_rooms);
            if dist == 0 {
                // In a room with fire! Good (can extinguish).
                score += 40.0;
            } else {
                // Move to fire
                score += (20.0 - dist as f64).max(0.0) * 5.0;
            }
        }

        // -- Role: Baker (Empty Hands) --
        // If we need ammo (boss HP > 0) and have space, go to Kitchen.
        if state.enemy.hp > 0 && p.inventory.len() < 2 && peppernuts == 0 && !has_extinguisher {
             let dist = min_distance(state, p.room_id, &kitchen_room_set);
             if dist == 0 {
                 score += 10.0;
             } else {
                 score += (10.0 - dist as f64).max(0.0) * 1.0;
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
