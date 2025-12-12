use clap::Parser;
use sint_core::logic::GameLogic;
use sint_solver::replay;
use sint_solver::scoring::ScoringWeights;
use sint_solver::search::{beam_search, SearchConfig};
use std::fs::File;
use std::io::Write;

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
    #[arg(short, long, default_value_t = 500)]
    width: usize,

    /// Max steps (actions/depth)
    #[arg(short, long, default_value_t = 3000)]
    steps: usize,

    /// Time limit in seconds
    #[arg(short, long, default_value_t = 300)]
    time_limit: u64,

    /// Output file for trajectory
    #[arg(short, long)]
    output: Option<String>,
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

        let initial_print = GameLogic::new_game(player_ids, args.seed);
        let output_lines = replay::format_trajectory(initial_print, sol.history);

        if let Some(path) = args.output {
            let mut file = File::create(&path).expect("Unable to create output file");
            let mut total_lines = 0;
            let mut current_round = 0;
            let mut boss_events = Vec::new();

            for chunk in output_lines {
                // Write the chunk to file
                write!(file, "{}", chunk).expect("Unable to write data");

                // Process the chunk line by line to track context
                for line in chunk.lines() {
                    total_lines += 1;

                    if line.contains("--- ROUND") {
                        // Format: "--- ROUND 12 ---"
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            if let Ok(r) = parts[2].parse::<usize>() {
                                current_round = r;
                            }
                        }
                    }

                    if line.contains("BOSS DEFEATED") {
                        // Format: "⚔️  BOSS DEFEATED: The Petty Thief  ⚔️"
                        let clean_line = line.replace("⚔️", "").trim().to_string();
                        boss_events.push(format!(
                            "Round {} (Line {}): {}",
                            current_round, total_lines, clean_line
                        ));
                    }
                }
            }

            println!("\n✅ Trajectory written to '{}'", path);
            println!("📊 Summary:");
            println!("  Total Rounds: {}", sol.state.turn_count);
            println!("  Total Lines: {}", total_lines);
            if !boss_events.is_empty() {
                println!("  Boss Defeats:");
                for event in boss_events {
                    println!("    - {}", event);
                }
            } else {
                println!("  Boss Defeats: 0");
            }
        } else {
            // Replay to stdout
            println!("\n📜 Trajectory:");
            for line in output_lines {
                print!("{}", line);
            }
        }
    } else {
        println!("❌ No solution found.");
    }
}
