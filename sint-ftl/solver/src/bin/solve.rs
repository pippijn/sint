use clap::Parser;
use rayon::prelude::*;
use sint_core::logic::GameLogic;
use sint_core::types::{Action, GameAction, GamePhase, GameState, PlayerId};
use sint_solver::replay;
use sint_solver::scoring::score_state;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of players
    #[arg(short, long, default_value_t = 6)]
    players: usize,

    /// Random Seed
    #[arg(short, long, default_value_t = 12345)]
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

#[derive(Clone)]
struct SearchNode {
    state: GameState,
    history: Vec<(PlayerId, GameAction)>,
    score: f64,
}

impl PartialEq for SearchNode {
    fn eq(&self, other: &Self) -> bool {
        // Approximate equality for dedup: check score and phase/turn
        self.score == other.score
            && self.state.turn_count == other.state.turn_count
            && self.state.phase == other.state.phase
            && self.state.hull_integrity == other.state.hull_integrity
    }
}
impl Eq for SearchNode {}
impl std::hash::Hash for SearchNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash key state properties for dedup
        ((self.score * 1000.0) as i64).hash(state);
        self.state.turn_count.hash(state);
        self.state.phase.hash(state);
        self.state.hull_integrity.hash(state);
        self.state.players.len().hash(state);
    }
}

fn get_state_signature(state: &GameState) -> u64 {
    let mut hasher = DefaultHasher::new();
    state.phase.hash(&mut hasher);
    state.turn_count.hash(&mut hasher);
    state.hull_integrity.hash(&mut hasher);
    state.enemy.hp.hash(&mut hasher);
    
    // Players (BTreeMap iterates sorted by key)
    for p in state.players.values() {
        p.room_id.hash(&mut hasher);
        p.hp.hash(&mut hasher);
        p.is_ready.hash(&mut hasher);
        p.inventory.hash(&mut hasher);
        p.status.hash(&mut hasher);
    }
    
    // Map
    for room in state.map.rooms.values() {
        room.hazards.hash(&mut hasher);
        room.items.hash(&mut hasher);
    }

    // Active Situations
    let mut sit_ids: Vec<_> = state.active_situations.iter().map(|c| c.id).collect();
    sit_ids.sort(); // Sort to ensure canonical order
    sit_ids.hash(&mut hasher);
    
    // Deck/Discard counts
    state.deck.len().hash(&mut hasher);
    state.discard.len().hash(&mut hasher);

    // Proposal Queue (CRITICAL: Must distinguish queued actions!)
    // GameAction doesn't impl Hash, so we hash the debug string which acts as a canonical representation.
    for prop in &state.proposal_queue {
        prop.hash(&mut hasher);
    }
    
    hasher.finish()
}

fn main() {
    let args = Args::parse();

    // Init
    let player_ids: Vec<String> = (0..args.players).map(|i| format!("P{}", i + 1)).collect();
    let initial_state = GameLogic::new_game(player_ids.clone(), args.seed);

    println!(
        "🚀 Starting Beam Search: Width={}, Seed={}, Players={}, Steps={}, TimeLimit={}s",
        args.width, args.seed, args.players, args.steps, args.time_limit
    );

    let start_time = Instant::now();
    let time_limit = Duration::from_secs(args.time_limit);

    let mut beam = vec![SearchNode {
        state: initial_state,
        history: Vec::new(),
        score: 0.0,
    }];

    let mut final_solution: Option<SearchNode> = None;
    // Map Signature -> Max AP seen
    let mut visited: HashMap<u64, i32> = HashMap::new();

    for step in 0..args.steps {
        if beam.is_empty() {
            println!("💀 Beam died at step {}", step);
            break;
        }

        if start_time.elapsed() > time_limit {
            println!("⏰ Time limit reached at step {}", step);
            break;
        }

        // Check for victory in current beam
        if let Some(win) = beam
            .iter()
            .find(|n| n.state.phase == GamePhase::Victory || n.state.phase == GamePhase::GameOver)
        {
            if win.state.phase == GamePhase::Victory {
                println!("🏆 VICTORY FOUND at step {}!", step);
                final_solution = Some(win.clone());
                break;
            }
        }

        // Parallel Expansion
        let next_nodes: Vec<SearchNode> = beam
            .par_iter()
            .flat_map(|node| expand_node(node))
            .collect();

        // Dedup & Prune
        let mut unique_nodes: HashMap<u64, SearchNode> = HashMap::with_capacity(next_nodes.len());
        for n in next_nodes {
            // Pruning by Dominance (Visited Check)
            let sig = get_state_signature(&n.state);
            let total_ap: i32 = n.state.players.values().map(|p| p.ap).sum();

            if let Some(&max_ap) = visited.get(&sig) {
                if max_ap >= total_ap {
                    continue; // Prune: We saw this state with equal or more AP before
                }
            }
            visited.insert(sig, total_ap);

            // Use state signature as key to preserve distinct states
            unique_nodes.insert(sig, n);
        }

        let mut sorted_nodes: Vec<SearchNode> = unique_nodes.into_values().collect();
        // Sort descending by score
        sorted_nodes.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Keep top K
        if sorted_nodes.len() > args.width {
            sorted_nodes.truncate(args.width);
        }
        beam = sorted_nodes;

        // Logging
        if step % 10 == 0 && !beam.is_empty() {
            let best = &beam[0];
            println!(
                "Step {}: Best Score {:.1} | Round {} | Phase {:?} | Hull {} | Boss {} | Beam {}",
                step,
                best.score,
                best.state.turn_count,
                best.state.phase,
                best.state.hull_integrity,
                best.state.enemy.hp,
                beam.len()
            );
        }
    }

    let elapsed = start_time.elapsed();
    println!("⏱️ Search finished in {:.2?}", elapsed);

    if let Some(sol) = final_solution.or_else(|| beam.first().cloned()) {
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

// Helper to fast-forward through deterministic phases
fn fast_forward_phase(mut state: GameState, mut history: Vec<(PlayerId, GameAction)>) -> (GameState, Vec<(PlayerId, GameAction)>) {
    let mut i = 0;
    while state.phase != GamePhase::TacticalPlanning 
        && state.phase != GamePhase::GameOver 
        && state.phase != GamePhase::Victory 
    {
        i += 1;
        if i > 1000 { break; } // Safety
        
        let mut ready_action = None;
        // Find first non-ready player (deterministically)
        for p in state.players.values() {
            if !p.is_ready {
                 ready_action = Some((p.id.clone(), GameAction::VoteReady { ready: true }));
                 break;
            }
        }
        
        if let Some((pid, act)) = ready_action {
            let core_act = Action::Game(act.clone());
            if let Ok(next) = GameLogic::apply_action(state.clone(), &pid, core_act, None) {
                state = next;
                history.push((pid, act));
            } else {
                break;
            }
        } else {
             break;
        }
    }
    (state, history)
}

fn expand_node(node: &SearchNode) -> Vec<SearchNode> {
    let state = &node.state;

    // 1. If in automated phase, advance automatically (Fast Forward)
    if state.phase != GamePhase::TacticalPlanning {
        if state.phase == GamePhase::GameOver || state.phase == GamePhase::Victory {
            return vec![node.clone()]; // Terminal
        }

        let (new_state, new_history) = fast_forward_phase(state.clone(), node.history.clone());
        
        let score = score_state(&new_state);
        return vec![SearchNode {
            state: new_state,
            history: new_history,
            score,
        }];
    }

    // 2. Find the FIRST active player (deterministic order)
    let active_player = state.players.values().find(|p| !p.is_ready && p.ap > 0);

    match active_player {
        Some(p) => {
            // Get all LEGAL atomic actions from Core
            // This handles AP, Location, Items, Cards, etc. automatically.
            let legal_actions = sint_core::logic::actions::get_valid_actions(state, &p.id);
            let mut results = Vec::new();

            for action_wrapper in legal_actions {
                if let Action::Game(act) = action_wrapper {
                    // Filter out meta-actions irrelevant for solver search
                    if matches!(act, GameAction::Undo { .. } | GameAction::VoteReady { .. }) {
                        continue;
                    }

                    let mut current_state = state.clone();
                    let mut current_history = node.history.clone();

                    // Apply the single atomic action
                    if let Ok(next) = GameLogic::apply_action(current_state.clone(), &p.id, Action::Game(act.clone()), None) {
                        current_state = next;
                        current_history.push((p.id.clone(), act));
                        
                        let score = score_state(&current_state);
                        results.push(SearchNode {
                            state: current_state,
                            history: current_history,
                            score,
                        });
                    }
                }
            }

            results
        }
        None => {
            // No players have AP. Everyone must Vote Ready to end the phase.
            let mut current_state = state.clone();
            let mut current_history = node.history.clone();
            
            // Loop until phase changes or all ready
            loop {
                if current_state.phase != GamePhase::TacticalPlanning { break; }
                
                let next_unready = current_state.players.values().find(|p| !p.is_ready);
                match next_unready {
                    Some(p) => {
                         let pid = p.id.clone();
                         let act = GameAction::VoteReady { ready: true };
                         if let Ok(next) = GameLogic::apply_action(current_state.clone(), &pid, Action::Game(act.clone()), None) {
                             current_state = next;
                             current_history.push((pid, act));
                         } else {
                             break; // Should not happen
                         }
                    }
                    None => break, // Everyone ready
                }
            }
            
            let score = score_state(&current_state);
            vec![SearchNode {
                state: current_state,
                history: current_history,
                score,
            }]
        }
    }
}

// Remove unused `apply_actions` helper since we inline the application loop now.
