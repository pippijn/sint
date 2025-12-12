use crate::driver::GameDriver;
use crate::scoring::{score_state, ScoringWeights};
use rayon::prelude::*;
use sint_core::logic::GameLogic;
use sint_core::types::{Action, GameAction, GamePhase, GameState, PlayerId};
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct SearchNode {
    pub state: GameState,
    pub history: Vec<(PlayerId, GameAction)>,
    pub score: f64,
    pub signature: u64,
}

impl PartialEq for SearchNode {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
            && self.state.turn_count == other.state.turn_count
            && self.state.phase == other.state.phase
            && self.state.hull_integrity == other.state.hull_integrity
    }
}
impl Eq for SearchNode {}
impl std::hash::Hash for SearchNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ((self.score * 1000.0) as i64).hash(state);
        self.state.turn_count.hash(state);
        self.state.phase.hash(state);
        self.state.hull_integrity.hash(state);
        self.state.players.len().hash(state);
    }
}

struct DeterministicHasher(u64);
impl Hasher for DeterministicHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= b as u64;
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
    }
}

fn get_state_signature(state: &GameState) -> u64 {
    let mut hasher = DeterministicHasher(0xcbf29ce484222325);
    state.phase.hash(&mut hasher);
    state.turn_count.hash(&mut hasher);
    state.hull_integrity.hash(&mut hasher);
    state.enemy.hp.hash(&mut hasher);
    state.enemy.next_attack.hash(&mut hasher);

    for p in state.players.values() {
        p.room_id.hash(&mut hasher);
        p.hp.hash(&mut hasher);
        p.ap.hash(&mut hasher);
        p.is_ready.hash(&mut hasher);
        p.inventory.hash(&mut hasher);
        p.status.hash(&mut hasher);
    }

    for room in state.map.rooms.values() {
        room.hazards.hash(&mut hasher);
        room.items.hash(&mut hasher);
    }

    // Hash active situations fully (sorted by ID for canonicalization)
    let mut situations: Vec<_> = state.active_situations.iter().collect();
    situations.sort_by_key(|c| c.id);
    for card in situations {
        card.hash(&mut hasher);
    }

    state.deck.len().hash(&mut hasher);
    state.discard.len().hash(&mut hasher);

    for prop in &state.proposal_queue {
        prop.hash(&mut hasher);
    }

    hasher.finish()
}

pub struct SearchConfig {
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
    pub actions_generated: Vec<String>,
    pub outcomes: Vec<String>,
}

pub struct DebugContext {
    pub logs: Mutex<VecDeque<ExpansionLog>>,
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
                println!("  Generated: {:?}", log.actions_generated);
                for out in &log.outcomes {
                    println!("  -> {}", out);
                }
                println!("------------------------------------------------");
            }
        }
    }
}

pub fn beam_search(config: &SearchConfig, weights: &ScoringWeights) -> Option<SearchNode> {
    let player_ids: Vec<String> = (0..config.players).map(|i| format!("P{}", i + 1)).collect();
    let initial_state = GameLogic::new_game(player_ids.clone(), config.seed);

    // Stabilize initial state using Driver
    let initial_driver = GameDriver::new(initial_state);

    if config.verbose {
        println!(
            "🚀 Starting Beam Search: Width={}, Seed={}, Players={}, Steps={}, TimeLimit={}s",
            config.width, config.seed, config.players, config.steps, config.time_limit
        );
    }

    let start_time = Instant::now();
    let time_limit = Duration::from_secs(config.time_limit);

    let start_sig = get_state_signature(&initial_driver.state);
    let mut beam = vec![SearchNode {
        state: initial_driver.state,
        history: Vec::new(),
        score: 0.0,
        signature: start_sig,
    }];

    let mut final_solution: Option<SearchNode> = None;
    let mut best_partial: Option<SearchNode> = beam.first().cloned();
    let mut visited: HashMap<u64, (i32, f64)> = HashMap::new();

    let debug_ctx = Arc::new(DebugContext::new());

    for step in 0..config.steps {
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
                    if best_in_beam.score > current_best.score {
                        best_partial = Some(best_in_beam.clone());
                    }
                }
            } else {
                best_partial = Some(best_in_beam.clone());
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
        {
            if win.state.phase == GamePhase::Victory {
                if config.verbose {
                    println!("🏆 VICTORY FOUND at step {}!", step);
                }
                final_solution = Some(win.clone());
                break;
            }
        }

        let debug_clone = debug_ctx.clone();
        let next_nodes: Vec<SearchNode> = beam
            .par_iter()
            .flat_map(|node| expand_node(node, weights, config, step, &debug_clone))
            .collect();

        let total_generated = next_nodes.len();

        let mut unique_nodes: std::collections::BTreeMap<u64, SearchNode> =
            std::collections::BTreeMap::new();
        for n in &next_nodes {
            let sig = n.signature;
            let total_ap: i32 = n.state.players.values().map(|p| p.ap).sum();

            if let Some(&(max_ap, max_score)) = visited.get(&sig) {
                if total_ap < max_ap {
                    continue;
                }
                if total_ap == max_ap && n.score <= max_score {
                    continue;
                }
            }
            visited.insert(sig, (total_ap, n.score));
            unique_nodes.insert(sig, n.clone());
        }

        let mut sorted_nodes: Vec<SearchNode> = unique_nodes.into_values().collect();
        sorted_nodes.par_sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        if sorted_nodes.is_empty() && !beam.is_empty() {
            if config.verbose {
                let best = &beam[0];
                let round_num = if step > 0 { step - 1 } else { 0 };
                println!(
                    "Step {} (Last Valid): Best Score {:.1} | Round {} | Phase {:?} | Hull {} | Boss {} | Beam {}",
                    round_num,
                    best.score,
                    best.state.turn_count,
                    best.state.phase,
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
                    println!("   Possible reasons: No legal actions, or all actions filtered (Undo/Chat/etc), or apply_action failed.");
                    debug_ctx.dump();
                } else {
                    println!("   Reason: All generated states were already visited with better/equal cost.");
                    // Print collision info if useful
                }
            }
        }

        if sorted_nodes.len() > config.width {
            sorted_nodes.truncate(config.width);
        }
        beam = sorted_nodes;

        if config.verbose && !beam.is_empty() {
            if step % 10 == 0 || step == config.steps - 1 {
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
    }

    if config.verbose {
        let elapsed = start_time.elapsed();
        println!("⏱️ Search finished in {:.2?}", elapsed);
    }

    final_solution.or(best_partial)
}

fn expand_node(
    node: &SearchNode,
    weights: &ScoringWeights,
    _config: &SearchConfig,
    step: usize,
    debug: &DebugContext,
) -> Vec<SearchNode> {
    let state = &node.state;

    // Driver guarantees we are in a stable state (TacticalPlanning or GameOver)

    if state.phase == GamePhase::GameOver || state.phase == GamePhase::Victory {
        return vec![node.clone()];
    }

    // Identify active player (Deterministic order: P1, P2, ...)
    let mut players: Vec<_> = state.players.values().collect();
    players.sort_by_key(|p| &p.id);
    let active_player = players.into_iter().find(|p| !p.is_ready && p.ap > 0);

    // Prepare log entry
    let mut log_entry = ExpansionLog {
        step,
        phase: state.phase.clone(),
        active_player: active_player.map(|p| p.id.clone()),
        ap: active_player.map(|p| p.ap).unwrap_or(0),
        actions_generated: Vec::new(),
        outcomes: Vec::new(),
    };

    match active_player {
        Some(p) => {
            let legal_actions = sint_core::logic::actions::get_valid_actions(state, &p.id);
            log_entry.actions_generated =
                legal_actions.iter().map(|a| format!("{:?}", a)).collect();

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
                            let mut current_history = node.history.clone();
                            current_history.push((p.id.clone(), act.clone()));
                            let score = score_state(&driver.state, &current_history, weights);
                            let signature = get_state_signature(&driver.state);
                            results.push(SearchNode {
                                state: driver.state,
                                history: current_history,
                                score,
                                signature,
                            });
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
            // No active players with AP > 0.
            // This implies stable state requires a transition, but Driver.stabilize()
            // guarantees that if we are in TacticalPlanning, there IS an unready player with AP > 0,
            // UNLESS all unready players have AP <= 0 (which Driver should have forced ready).

            // If we hit this, it means we are "stuck" in TacticalPlanning with everyone Ready (wait, no, Driver advances phase then)
            // or everyone who is Unready has 0 AP (Driver advances them).

            // So if we return None here, it means we are in a state not covered by Driver logic,
            // or Driver returned a state where everyone is Ready but phase didn't change?

            // Log it
            log_entry
                .outcomes
                .push("No Active Player found in ostensibly stable state".to_string());
            debug.log(log_entry);

            vec![]
        }
    }
}
