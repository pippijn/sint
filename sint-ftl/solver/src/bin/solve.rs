use clap::Parser;
use sint_core::logic::GameLogic;
use sint_solver::replay;
use sint_solver::scoring::ScoringWeights;
use sint_solver::search::{beam_search, SearchConfig};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of players
    #[arg(short, long, default_value_t = 6)]
    players: usize,

    /// Random Seed
    #[arg(long, default_value_t = 12345)]
    seed: u64,

    /// Beam Width (Number of states to keep per step)
    #[arg(short, long, default_value_t = 100)]
    width: usize,

    /// Max steps (actions/depth)
    #[arg(short, long, default_value_t = 500)]
    steps: usize,

    /// Time limit in seconds
    #[arg(short, long, default_value_t = 60)]
    time_limit: u64,
}

fn main() {
    let args = Args::parse();

    // Init
    let player_ids: Vec<String> = (0..args.players).map(|i| format!("P{}", i + 1)).collect();

    // Load Scoring Weights
    let weights = ScoringWeights::default();

    let config = SearchConfig {
        players: args.players,
        seed: args.seed,
        width: args.width,
        steps: args.steps,
        time_limit: args.time_limit,
        verbose: true,
    };

    if let Some(sol) = beam_search(&config, &weights) {
        println!("\n=== BEST RESULT ===");
        println!("Final Phase: {:?}", sol.state.phase);
        println!("Hull: {}", sol.state.hull_integrity);
        println!("Boss HP: {}", sol.state.enemy.hp);

        // Replay
        println!("\n📜 Trajectory:");
        let initial_print = GameLogic::new_game(player_ids, args.seed);
        replay::print_trajectory(initial_print, sol.history);
    } else {
        println!("❌ No solution found.");
    }
}
