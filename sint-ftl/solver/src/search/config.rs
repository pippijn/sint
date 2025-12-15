use clap::Parser;

#[derive(Parser, Debug, Clone, Copy)]
pub struct CommonSearchConfig {
    /// Number of players
    #[arg(short, long, default_value_t = 6)]
    pub players: usize,

    /// Random Seed
    #[arg(long, default_value_t = 12345)]
    pub seed: u64,

    /// Max steps (actions/depth)
    #[arg(short, long, default_value_t = 3000)]
    pub steps: usize,

    /// Time limit in seconds
    #[arg(short, long, default_value_t = 300)]
    pub time_limit: u64,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Parser, Debug, Clone, Copy)]
pub struct BeamConfig {
    /// Beam Width (Number of states to keep per step)
    #[arg(long, default_value_t = 100)]
    pub beam_width: usize,
}

#[derive(Parser, Debug, Clone, Copy)]
pub struct RHEAConfigParams {
    /// RHEA Horizon
    #[arg(long, default_value_t = 30)]
    pub rhea_horizon: usize,

    /// RHEA Generations per step
    #[arg(long, default_value_t = 10)]
    pub rhea_generations: usize,

    /// RHEA Population
    #[arg(long, default_value_t = 10)]
    pub rhea_population: usize,
}

pub struct BeamSearchConfig {
    pub players: usize,
    pub seed: u64,
    pub width: usize,
    pub steps: usize,
    pub time_limit: u64,
    pub verbose: bool,
}

pub struct RHEAConfig {
    pub players: usize,
    pub seed: u64,
    pub horizon: usize,
    pub generations: usize,
    pub population_size: usize,
    pub max_steps: usize,
    pub time_limit: u64,
    pub verbose: bool,
}

impl Default for CommonSearchConfig {
    fn default() -> Self {
        Self {
            players: 6,
            seed: 12345,
            steps: 3000,
            time_limit: 300,
            verbose: false,
        }
    }
}

impl Default for BeamConfig {
    fn default() -> Self {
        Self { beam_width: 300 }
    }
}

impl Default for RHEAConfigParams {
    fn default() -> Self {
        Self {
            rhea_horizon: 30,
            rhea_generations: 10,
            rhea_population: 20,
        }
    }
}
