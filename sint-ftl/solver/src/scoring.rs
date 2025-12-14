pub mod beam;
pub mod rhea;
pub mod rl;

/// Detailed breakdown of the scoring components for debugging and analysis.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ScoreDetails {
    pub total: f64,
    pub vitals: f64,           // Hull, Player HP, Victory/GameOver
    pub hazards: f64,          // Fire, Water, Systems
    pub offense: f64,          // Enemy HP, Shooting
    pub panic: f64,            // Critical state penalties
    pub logistics: f64,        // Ammo, Station keeping, Scavenging
    pub situations: f64,       // Active situations and solutions
    pub threats: f64,          // Pending enemy attacks
    pub progression: f64,      // Boss level, Turns, Steps
    pub anti_oscillation: f64, // Backtracking penalties
}

impl PartialOrd for ScoreDetails {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.total.partial_cmp(&other.total)
    }
}

impl std::ops::AddAssign for ScoreDetails {
    fn add_assign(&mut self, rhs: Self) {
        self.total += rhs.total;
        self.vitals += rhs.vitals;
        self.hazards += rhs.hazards;
        self.offense += rhs.offense;
        self.panic += rhs.panic;
        self.logistics += rhs.logistics;
        self.situations += rhs.situations;
        self.threats += rhs.threats;
        self.progression += rhs.progression;
        self.anti_oscillation += rhs.anti_oscillation;
    }
}

impl ScoreDetails {
    pub fn format_short(&self) -> String {
        format!(
            "Total: {:.0} [V: {:.0}, H: {:.0}, O: {:.0}, P: {:.0}, L: {:.0}, S: {:.0}, T: {:.0}, Pr: {:.0}, AO: {:.0}]",
            self.total,
            self.vitals,
            self.hazards,
            self.offense,
            self.panic,
            self.logistics,
            self.situations,
            self.threats,
            self.progression,
            self.anti_oscillation
        )
    }
}
