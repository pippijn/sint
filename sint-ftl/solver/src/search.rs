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

    if config.verbose {
        println!(
            "🚀 Starting Beam Search: Width={}, Seed={}, Players={}, Steps={}, TimeLimit={}s",
            config.width, config.seed, config.players, config.steps, config.time_limit
        );
    }

    let start_time = Instant::now();
    let time_limit = Duration::from_secs(config.time_limit);

    let mut beam = vec![SearchNode {
        state: initial_state,
        history: Vec::new(),
        score: 0.0,
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

        let mut unique_nodes: std::collections::BTreeMap<u64, SearchNode> =
            std::collections::BTreeMap::new();
        for n in next_nodes {
            let sig = get_state_signature(&n.state);
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
            unique_nodes.insert(sig, n);
        }

        let mut sorted_nodes: Vec<SearchNode> = unique_nodes.into_values().collect();
        sorted_nodes.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

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

fn fast_forward_phase(
    mut state: GameState,
    mut history: Vec<(PlayerId, GameAction)>,
) -> (GameState, Vec<(PlayerId, GameAction)>) {
    let mut i = 0;
    while state.phase != GamePhase::TacticalPlanning
        && state.phase != GamePhase::GameOver
        && state.phase != GamePhase::Victory
    {
        i += 1;
        if i > 1000 {
            break;
        }

        let mut ready_action = None;
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

fn expand_node(
    node: &SearchNode,
    weights: &ScoringWeights,
    _config: &SearchConfig,
    step: usize,
    debug: &DebugContext,
) -> Vec<SearchNode> {
    let state = &node.state;

    // If not in planning phase, fast forward until we are (or game ends)
    if state.phase != GamePhase::TacticalPlanning {
        if state.phase == GamePhase::GameOver || state.phase == GamePhase::Victory {
            return vec![node.clone()];
        }
        let (new_state, new_history) = fast_forward_phase(state.clone(), node.history.clone());

        if new_state.phase == GamePhase::GameOver || new_state.phase == GamePhase::Victory {
            let score = score_state(&new_state, &new_history, weights);
            return vec![SearchNode {
                state: new_state,
                history: new_history,
                score,
            }];
        }

        let score = score_state(&new_state, &new_history, weights);
        return vec![SearchNode {
            state: new_state,
            history: new_history,
            score,
        }];
    }

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
                            | GameAction::VoteReady { .. }
                            | GameAction::Chat { .. }
                    ) {
                        // log_entry.outcomes.push(format!("Skipped (Filter): {:?}", act));
                        continue;
                    }

                    match GameLogic::apply_action(
                        state.clone(),
                        &p.id,
                        Action::Game(act.clone()),
                        None,
                    ) {
                        Ok(next) => {
                            let mut current_history = node.history.clone();
                            current_history.push((p.id.clone(), act.clone()));
                            let score = score_state(&next, &current_history, weights);
                            results.push(SearchNode {
                                state: next,
                                history: current_history,
                                score,
                            });
                            // log_entry.outcomes.push(format!("Success: {:?}", act));
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
            // Logic for when no player is active (shouldn't happen if phase check works, but handled anyway)
            let mut current_state = state.clone();
            let mut current_history = node.history.clone();

            loop {
                if current_state.phase != GamePhase::TacticalPlanning {
                    break;
                }
                let next_unready = current_state.players.values().find(|p| !p.is_ready);
                match next_unready {
                    Some(p) => {
                        let pid = p.id.clone();
                        let act = GameAction::VoteReady { ready: true };
                        if let Ok(next) = GameLogic::apply_action(
                            current_state.clone(),
                            &pid,
                            Action::Game(act.clone()),
                            None,
                        ) {
                            current_state = next;
                            current_history.push((pid, act));
                        } else {
                            break;
                        }
                    }
                    None => break,
                }
            }

            let score = score_state(&current_state, &current_history, weights);
            vec![SearchNode {
                state: current_state,
                history: current_history,
                score,
            }]
        }
    }
}
