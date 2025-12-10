use sint_core::types::{GamePhase, GameState};

/// Calculates a score for a single state snapshot.
/// This is useful for "Greedy" evaluation or final state assessment.
pub fn score_state(state: &GameState) -> f64 {
    let mut score = 0.0;

    // 1. Hull Integrity (Base Health)
    // Range: 0-20. Weight: High.
    score += state.hull_integrity as f64 * 100.0;

    // 2. Boss Progress (Lower HP is better)
    score -= state.enemy.hp as f64 * 10.0;

    // 3. Hazard Penalty
    // Fewer hazards is better.
    let hazard_count: usize = state.map.rooms.values().map(|r| r.hazards.len()).sum();
    score -= hazard_count as f64 * 20.0;

    // 4. Survival (Round Count)
    // Later rounds = good (if surviving).
    score += state.turn_count as f64 * 50.0;

    if state.phase == GamePhase::Victory {
        score += 100_000.0;
    }

    score
}

/// Accumulator for trajectory scoring.
/// Tracks the "Area Under the Curve" for Hull and Hazards.
#[derive(Debug, Default, Clone)]
pub struct ScoreAccumulator {
    pub total_hull_integral: f64,
    pub total_hazard_integral: f64,
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

        // 4. Round Bonus (If not victory, strictly better to live longer)
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
