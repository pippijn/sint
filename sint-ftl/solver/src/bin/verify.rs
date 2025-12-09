use clap::Parser;
use sint_core::GameError;
use sint_core::logic::GameLogic;
use sint_core::types::{Action, GamePhase, GameState, ItemType};
use solver::replay;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the solution text file
    #[arg(short, long)]
    file: PathBuf,
}

fn main() {
    let args = Args::parse();

    // Read File first to check for overrides
    let (file_actions, file_seed, file_players) = parse_solution_file(&args.file);

    let seed = file_seed.unwrap();
    let players = file_players.unwrap();

    // Initialize Game
    let player_ids: Vec<String> = (0..players).map(|i| format!("P{}", i + 1)).collect();
    let initial_state = GameLogic::new_game(player_ids.clone(), seed);

    // Run
    println!("Verifying solution from {:?} with {} players, seed {}", args.file, players, seed);
    let final_history = run_verification(initial_state, file_actions);
    
    // Replay for visualization
    let state_for_print = GameLogic::new_game(player_ids, seed);
    replay::print_trajectory(state_for_print, final_history);
}

fn run_verification(mut state: GameState, user_actions: Vec<(String, Action)>) -> Vec<(String, Action)> {
    let mut full_history = Vec::new();
    let mut action_iter = user_actions.into_iter();
    
    // Loop until Game Over or Actions exhausted
    loop {
        if state.phase == GamePhase::GameOver || state.phase == GamePhase::Victory {
            break;
        }

        // Auto-Advance Phases
        if state.phase != GamePhase::TacticalPlanning {
             // Just Vote Ready for everyone
             let pids: Vec<String> = state.players.keys().cloned().collect();
             for pid in pids {
                 let act = Action::VoteReady { ready: true };
                 match GameLogic::apply_action(state.clone(), &pid, act.clone(), None) {
                     Ok(s) => {
                         state = s;
                         full_history.push((pid, act));
                     }
                     Err(_) => break, // Should not happen
                 }
             }
             continue;
        }

        if let Some((pid, action)) = action_iter.next() {
            // println!("Applying {}: {:?}", pid, action); // Optional verbose debug
            match GameLogic::apply_action(state.clone(), &pid, action.clone(), None) {
                Ok(mut s) => {
                    full_history.push((pid.clone(), action));

                    // Auto-Ready Logic
                    if s.phase == GamePhase::TacticalPlanning {
                        if let Some(p) = s.players.get(&pid) {
                            if p.ap == 0 && !p.is_ready {
                                let ready_act = Action::VoteReady { ready: true };
                                match GameLogic::apply_action(s.clone(), &pid, ready_act.clone(), None) {
                                    Ok(next_s) => {
                                        s = next_s;
                                        full_history.push((pid.clone(), ready_act));
                                    },
                                    Err(e) => println!("Auto-ready failed for {}: {}", pid, e),
                                }
                            }
                        }
                    }
                    state = s;
                }
                Err(e) => {
                    println!("Error applying {}: {:?} -> {}", pid, action, e);
                    print_failure_summary(&state, &pid, &action, &e);
                    break;
                }
            }
        } else {
            // No more user actions
            break;
        }
    }
    
    full_history
}

fn print_failure_summary(state: &GameState, pid: &str, action: &Action, error: &GameError) {
    println!("\n=== FAILURE SUMMARY ===");
    println!("Round: {}", state.turn_count);
    println!("Phase: {:?}", state.phase);
    println!("Failed Action: {} performs {:?}", pid, action);
    println!("Error: {}", error);
    
    println!("\n-- State Context --");
    println!("Hull: {} | Enemy: {} ({} HP)", state.hull_integrity, state.enemy.name, state.enemy.hp);
    
    println!("Active Situations:");
    for card in &state.active_situations {
        println!("  - {} ({:?})", card.title, card.id);
    }
    
    println!("Players:");
    let mut pids: Vec<String> = state.players.keys().cloned().collect();
    pids.sort();
    for p_id in pids {
        if let Some(p) = state.players.get(&p_id) {
            println!("  {}: Room {} | AP {} | HP {} | Inv {:?} | Status {:?}", 
                p_id, p.room_id, p.ap, p.hp, p.inventory, p.status);
        }
    }
    println!("=======================\n");
}

fn parse_solution_file(path: &PathBuf) -> (Vec<(String, Action)>, Option<u64>, Option<usize>) {
    let file = File::open(path).expect("Could not open solution file");
    let reader = io::BufReader::new(file);
    let mut actions = Vec::new();
    let mut seed = None;
    let mut players = None;

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Check for Meta Commands: "SEED 12345" or "PLAYERS 2"
        if line.starts_with("SEED ") {
            if let Ok(val) = line.replace("SEED ", "").trim().parse::<u64>() {
                seed = Some(val);
                continue;
            }
        }
        if line.starts_with("PLAYERS ") {
            if let Ok(val) = line.replace("PLAYERS ", "").trim().parse::<usize>() {
                players = Some(val);
                continue;
            }
        }

        let parts: Vec<&str> = line.splitn(2, ':').collect();
        if parts.len() != 2 {
            eprintln!("Warning: Skipping invalid line: {}", line);
            continue;
        }
        
        let pid = parts[0].trim().to_string();
        let cmd = parts[1].trim();

        let action = if cmd.starts_with("Move") {
            let target: u32 = cmd.split_whitespace().nth(1).unwrap().parse().unwrap();
            Action::Move { to_room: target }
        } else if cmd == "Bake" {
            Action::Bake
        } else if cmd == "Shoot" {
            Action::Shoot
        } else if cmd.starts_with("Throw") {
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            let target = parts[1].to_string();
            let idx = parts[2].parse().unwrap();
            Action::Throw { target_player: target, item_index: idx }
        } else if cmd == "Extinguish" {
            Action::Extinguish
        } else if cmd == "Repair" {
            Action::Repair
        } else if cmd == "PickUp" {
             Action::PickUp { item_type: ItemType::Peppernut }
        } else if cmd.starts_with("Drop") {
            let idx = cmd.split_whitespace().nth(1).unwrap().parse().unwrap();
            Action::Drop { item_index: idx }
        } else if cmd == "Pass" {
            Action::Pass
        } else if cmd == "Ready" {
            Action::VoteReady { ready: true }
        } else if cmd == "RaiseShields" {
            Action::RaiseShields
        } else if cmd == "EvasiveManeuvers" {
            Action::EvasiveManeuvers
        } else if cmd == "Lookout" {
            Action::Lookout
        } else if cmd == "Interact" {
            Action::Interact
        } else if cmd.starts_with("Revive") {
             let target = cmd.split_whitespace().nth(1).unwrap().to_string();
             Action::Revive { target_player: target }
        } else if cmd.starts_with("FirstAid") {
             let target = cmd.split_whitespace().nth(1).unwrap().to_string();
             Action::FirstAid { target_player: target }
        } else if cmd.starts_with("Chat") {
            let msg = cmd.replace("Chat ", "").trim().to_string();
            Action::Chat { message: msg }
        } else {
            panic!("Unknown command: {}", cmd);
        };

        actions.push((pid, action));
    }
    (actions, seed, players)
}
