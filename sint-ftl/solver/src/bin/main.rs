use clap::Parser;
use log::info;
use sint_core::logic::GameLogic;
use solver::{beam_search, replay, tui};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of players
    #[arg(short, long, default_value_t = 3)]
    players: usize,

    /// Random seed
    #[arg(short, long, default_value_t = 12345)]
    seed: u64,

    /// Beam width (number of states to keep per depth)
    #[arg(short, long, default_value_t = 100)]
    width: usize,

    /// Run in headless mode (text output only)
    #[arg(long)]
    headless: bool,
}

fn main() {
    let args = Args::parse();

    if args.headless {
        env_logger::init();
    }

    // Initialize Game
    let player_ids: Vec<String> = (0..args.players).map(|i| format!("P{}", i + 1)).collect();
    let initial_state = GameLogic::new_game(player_ids, args.seed);

    info!(
        "Starting solver for {} players, seed {}",
        args.players, args.seed
    );

    // Shared Search State
    let search_state = Arc::new(Mutex::new(beam_search::SearchStatus::new(
        initial_state.clone(),
    )));

    // Launch Search Thread
    let search_state_clone = search_state.clone();
    let width = args.width;
    let initial_state_clone = initial_state.clone();

    thread::spawn(move || {
        beam_search::run(initial_state_clone, width, search_state_clone);
    });

    if args.headless {
        // Simple loop to print status
        loop {
            thread::sleep(std::time::Duration::from_secs(2));
            let status = search_state.lock().unwrap();
            println!(
                "Depth: {}, Best Score: {}, Nodes: {}",
                status.depth, status.best_score, status.nodes_visited
            );

            // Debug: Print end of path
            if let Some(node) = &status.best_node {
                println!("Last actions:");
                for (pid, act) in node.path.iter().rev().take(5) {
                    println!("  {}: {:?}", pid, act);
                }
            }

            if status.finished {
                println!("Search Finished!");
                if let Some(node) = &status.best_node {
                    replay::print_trajectory(initial_state, node.path.clone());
                }
                break;
            }
        }
    } else {
        // Launch TUI
        tui::run(search_state.clone());

        // After TUI exit, print path
        let status = search_state.lock().unwrap();
        if let Some(node) = &status.best_node {
            replay::print_trajectory(initial_state, node.path.clone());
        }
    }
}
