use crate::driver::GameDriver;
use crate::scoring::ScoreDetails;
use crate::scoring::beam::{BeamScoringWeights, calculate_score};
use crate::search::{SearchNode, SearchProgress, get_state_signature};
use rayon::prelude::*;
use sint_core::logic::pathfinding::MapDistances;
use sint_core::logic::{GameLogic, actions::get_valid_actions};
use sint_core::types::{Action, GameAction, GamePhase};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct BeamSearchConfig {
    pub players: usize,
    pub seed: u64,
    pub width: usize,
    pub steps: usize,
    pub time_limit: u64,
    pub verbose: bool,
}

// --- DEBUG CONTEXT ---
#[derive(Debug, Clone)]
pub struct ExpansionLog {
    pub step: usize,
    pub phase: GamePhase,
    pub active_player: Option<String>,
    pub ap: i32,
    pub actions_generated: Vec<Action>,
    pub outcomes: Vec<String>,
}

pub struct DebugContext {
    pub logs: Mutex<VecDeque<ExpansionLog>>,
}

impl Default for DebugContext {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugContext {
    pub fn new() -> Self {
        Self {
            logs: Mutex::new(VecDeque::with_capacity(50)), // Keep last 50
        }
    }

    pub fn log(&self, entry: ExpansionLog) {
        let mut logs = self.logs.lock().unwrap();
        if logs.len() >= 50 {
            logs.pop_front();
        }
        logs.push_back(entry);
    }

    pub fn dump(&self) {
        let logs = self.logs.lock().unwrap();
        if !logs.is_empty() {
            println!("\n=== CRASH DUMP: Last 50 Expansions ===");
            for log in logs.iter() {
                println!(
                    "Step {}: Phase {:?} | Active: {:?} (AP: {})",
                    log.step, log.phase, log.active_player, log.ap
                );
                let actions_str: Vec<String> = log
                    .actions_generated
                    .iter()
                    .map(|a| format!("{:?}", a))
                    .collect();
                println!("  Generated: {:?}", actions_str);
                for out in &log.outcomes {
                    println!("  -> {}", out);
                }
                println!("------------------------------------------------");
            }
        }
    }
}

pub fn beam_search<F>(
    config: &BeamSearchConfig,
    weights: &BeamScoringWeights,
    progress_callback: Option<F>,
) -> Option<SearchNode>
where
    F: Fn(SearchProgress) + Sync + Send,
{
    let player_ids: Vec<String> = (0..config.players).map(|i| format!("P{}", i + 1)).collect();
    let initial_state = GameLogic::new_game(player_ids.clone(), config.seed);

    // Stabilize initial state using Driver
    let initial_driver = GameDriver::new(initial_state);

    let distances = Arc::new(MapDistances::new(&initial_driver.state.map));

    if config.verbose {
        println!(
            "🚀 Starting Beam Search: Width={}, Seed={}, Players={}, Steps={}, TimeLimit={}s",
            config.width, config.seed, config.players, config.steps, config.time_limit
        );
    }

    let start_time = Instant::now();
    let time_limit = Duration::from_secs(config.time_limit);

    let start_sig = get_state_signature(&initial_driver.state);
    let mut beam = vec![Arc::new(SearchNode {
        state: initial_driver.state,
        parent: None,
        last_action: None,
        score: ScoreDetails::default(),
        signature: start_sig,
        history_len: 0,
    })];

    let mut final_solution: Option<Arc<SearchNode>> = None;
    let mut best_partial: Option<Arc<SearchNode>> = beam.first().cloned();
    let mut visited: HashMap<u64, (i32, f64)> = HashMap::new();

    let debug_ctx = Arc::new(DebugContext::new());

    let mut last_step = 0;
    for step in 0..config.steps {
        last_step = step;
        if beam.is_empty() {
            if config.verbose {
                println!("💀 Beam died at step {}", step);
                debug_ctx.dump(); // DUMP LOGS ON DEATH
            }
            break;
        }

        // Update best_partial with the longest node (deepest in turns)
        if let Some(best_in_beam) = beam.first() {
            if let Some(current_best) = &best_partial {
                // Prioritize deeper turn count first
                if best_in_beam.state.turn_count > current_best.state.turn_count {
                    best_partial = Some(best_in_beam.clone());
                } else if best_in_beam.state.turn_count == current_best.state.turn_count {
                    // If turn count is same, keep the better score
                    if best_in_beam.score.total > current_best.score.total {
                        best_partial = Some(best_in_beam.clone());
                    }
                }
            } else {
                best_partial = Some(best_in_beam.clone());
            }

            // Report progress every step
            if let Some(cb) = &progress_callback {
                cb(SearchProgress {
                    step,
                    is_done: false,
                    failed: false,
                    node: best_in_beam.clone(),
                });
            }
        }

        if start_time.elapsed() > time_limit {
            if config.verbose {
                println!("⏰ Time limit reached at step {}", step);
            }
            break;
        }

        if let Some(win) = beam
            .iter()
            .find(|n| n.state.phase == GamePhase::Victory || n.state.phase == GamePhase::GameOver)
            && win.state.phase == GamePhase::Victory
        {
            if config.verbose {
                println!("🏆 VICTORY FOUND at step {}!", step);
            }
            final_solution = Some(win.clone());
            // Report final success
            if let Some(cb) = &progress_callback {
                cb(SearchProgress {
                    step,
                    is_done: true,
                    failed: false,
                    node: win.clone(),
                });
            }
            break;
        }

        let debug_clone = debug_ctx.clone();
        let distances_clone = distances.clone();
        let next_nodes: Vec<Arc<SearchNode>> = beam
            .par_iter()
            .flat_map(|node| {
                expand_node(
                    node.clone(),
                    weights,
                    config,
                    step,
                    &debug_clone,
                    &distances_clone,
                )
            })
            .collect();

        let total_generated = next_nodes.len();

        let mut unique_nodes: std::collections::BTreeMap<u64, Arc<SearchNode>> =
            std::collections::BTreeMap::new();
        for n in &next_nodes {
            let sig = n.signature;
            let total_ap: i32 = n.state.players.values().map(|p| p.ap).sum();

            if let Some(&(max_ap, max_score)) = visited.get(&sig) {
                // If we found a path with strictly more AP, it's better (more potential).
                // If same AP, only keep if score is strictly better.
                if total_ap < max_ap {
                    continue;
                }
                if total_ap == max_ap && n.score.total <= max_score {
                    continue;
                }
            }
            visited.insert(sig, (total_ap, n.score.total));
            unique_nodes.insert(sig, n.clone());
        }

        let mut sorted_nodes: Vec<Arc<SearchNode>> = unique_nodes.into_values().collect();
        sorted_nodes.par_sort_by(|a, b| {
            b.score
                .total
                .partial_cmp(&a.score.total)
                .unwrap()
                .then_with(|| a.signature.cmp(&b.signature))
        });

        if sorted_nodes.is_empty() && !beam.is_empty() && config.verbose {
            let best = &beam[0];
            let round_num = if step > 0 { step - 1 } else { 0 };
            println!(
                "Step {} (Last Valid): Best Score {:.1} | Round {} | Hull {} | Boss {} | Beam {}",
                round_num,
                best.score.total,
                best.state.turn_count,
                best.state.hull_integrity,
                best.state.enemy.hp,
                beam.len()
            );

            let kept_nodes = sorted_nodes.len(); // 0 in this block

            println!(
                "💀 Beam died! Generated {} nodes, kept {} (filtered by visited).",
                total_generated, kept_nodes
            );
            if total_generated == 0 {
                println!(
                    "   Possible reasons: No legal actions, or all actions filtered (Undo/Chat/etc), or apply_action failed."
                );
                debug_ctx.dump();
            } else {
                println!(
                    "   Reason: All generated states were already visited with better/equal cost."
                );
                // Print collision info if useful
            }
        }

        if sorted_nodes.len() > config.width {
            sorted_nodes.truncate(config.width);
        }
        beam = sorted_nodes;

        if let Some(cb) = beam.first()
            && config.verbose
            && step % 10 == 0
        {
            println!(
                "Step {}: Best Score {:.1} | Round {} | Hull {} | Boss {} | Beam {}",
                step,
                cb.score.total,
                cb.state.turn_count,
                cb.state.hull_integrity,
                cb.state.enemy.hp,
                beam.len()
            );
            println!("  Score Breakdown: {}", cb.score.format_short());
        }
    }

    if config.verbose {
        let elapsed = start_time.elapsed();
        println!("⏱️ Search finished in {:.2?}", elapsed);
    }

    let result = final_solution.clone().or(best_partial);

    // Report final progress to signal completion (especially if beam died or time limit reached)
    if let Some(cb) = &progress_callback
        && let Some(node) = &result
    {
        cb(SearchProgress {
            step: last_step, // Report the actual step reached
            is_done: true,
            failed: final_solution.is_none() && beam.is_empty(),
            node: node.clone(),
        });
    }

    result.map(|n| (*n).clone())
}

fn expand_node(
    node: Arc<SearchNode>,
    weights: &BeamScoringWeights,
    _config: &BeamSearchConfig,
    step: usize,
    debug: &DebugContext,
    distances: &MapDistances,
) -> Vec<Arc<SearchNode>> {
    let state = &node.state;

    // Driver guarantees we are in a stable state (TacticalPlanning or GameOver)

    if state.phase == GamePhase::GameOver || state.phase == GamePhase::Victory {
        return vec![node.clone()];
    }

    // Identify active player (Deterministic order: P1, P2, ...)
    let players: Vec<_> = state.players.values().collect();
    let active_player = players.into_iter().find(|p| !p.is_ready && p.ap > 0);

    // Prepare log entry
    let mut log_entry = ExpansionLog {
        step,
        phase: state.phase,
        active_player: active_player.map(|p| p.id.clone()),
        ap: active_player.map(|p| p.ap).unwrap_or(0),
        actions_generated: Vec::new(),
        outcomes: Vec::new(),
    };

    match active_player {
        Some(p) => {
            let legal_actions = get_valid_actions(state, &p.id);
            log_entry.actions_generated = legal_actions.clone();

            let mut results = Vec::new();

            for action_wrapper in &legal_actions {
                if let Action::Game(act) = action_wrapper {
                    if matches!(
                        act,
                        GameAction::Undo { .. }
                            | GameAction::VoteReady { .. } // Driver handles readiness
                            | GameAction::Chat { .. }
                            | GameAction::Pass // Pass is allowed if AP > 0 to skip turn
                    ) {
                        if matches!(act, GameAction::Pass) {
                            // Keep Pass
                        } else {
                            // Filter others
                            continue;
                        }
                    }

                    // Apply using Driver
                    let mut driver = GameDriver {
                        state: state.clone(),
                    };
                    match driver.apply(&p.id, act.clone()) {
                        Ok(_) => {
                            let next_action = (p.id.clone(), act.clone());
                            // Limit lookback for scoring to 48 steps to avoid O(N^2) complexity
                            let mut current_history = node.get_recent_history(47);
                            current_history.push(&next_action);

                            let score = calculate_score(
                                &node.state,
                                &driver.state,
                                &current_history,
                                weights,
                                distances,
                            );

                            let signature = get_state_signature(&driver.state);
                            results.push(Arc::new(SearchNode {
                                state: driver.state,
                                parent: Some(node.clone()),
                                last_action: Some(next_action),
                                score,
                                signature,
                                history_len: node.history_len + 1,
                            }));
                        }
                        Err(e) => {
                            log_entry
                                .outcomes
                                .push(format!("Failed {:?}: {:?}", act, e));
                        }
                    }
                }
            }

            // Only log if we failed to produce children
            if results.is_empty() {
                debug.log(log_entry);
            }

            results
        }
        None => {
            // No Active Player found in ostensibly stable state
            log_entry
                .outcomes
                .push("No Active Player found in ostensibly stable state".to_string());
            debug.log(log_entry);

            vec![]
        }
    }
}
