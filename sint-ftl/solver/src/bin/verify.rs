use clap::Parser;
use sint_core::logic::GameLogic;
use sint_solver::replay;
use sint_solver::verification::{parse_solution_text, run_verification};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the solution text file
    #[arg(short, long)]
    file: PathBuf,

    /// Print full trajectory even on failure
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    // Read File
    let content = fs::read_to_string(&args.file).expect("Could not read solution file");

    // Parse
    let (file_actions, file_seed, file_players) = parse_solution_text(&content);

    let seed = file_seed.expect("Solution file must specify SEED");
    let players = file_players.expect("Solution file must specify PLAYERS");

    // Initialize Game
    let player_ids: Vec<String> = (0..players).map(|i| format!("P{}", i + 1)).collect();
    let initial_state = GameLogic::new_game(player_ids.clone(), seed);

    // Run
    println!(
        "Verifying solution from {:?} with {} players, seed {}",
        args.file, players, seed
    );

    let result = run_verification(initial_state, file_actions);

    if !result.success {
        if let Some(summary) = result.failure_summary() {
            println!("{}", summary);
        }
    }

    // Replay for visualization
    if result.success || args.verbose {
        let state_for_print = GameLogic::new_game(player_ids, seed);
        replay::print_trajectory(state_for_print, result.history);
    }
}
