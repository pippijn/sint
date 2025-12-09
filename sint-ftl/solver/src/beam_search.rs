use crate::heuristic;
use crate::macro_actions::{generate_tasks, PlayerTask};
use crate::state_node::StateNode;
use log::info;
use rayon::prelude::*;
use sint_core::logic::GameLogic;
use sint_core::types::{Action, GamePhase, GameState};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct SearchStatus {
    pub best_node: Option<StateNode>,
    pub best_score: i32,
    pub nodes_visited: usize,
    pub depth: usize,
    pub finished: bool,
    pub current_frontier_size: usize,
}

impl SearchStatus {
    pub fn new(_initial_state: GameState) -> Self {
        SearchStatus {
            best_node: None,
            best_score: i32::MIN,
            nodes_visited: 0,
            depth: 0,
            finished: false,
            current_frontier_size: 1,
        }
    }
}

pub fn run(initial_state: GameState, beam_width: usize, status: Arc<Mutex<SearchStatus>>) {
    let mut frontier = vec![StateNode::new(initial_state)];

    // Initial Score
    frontier[0].score = heuristic::evaluate(&frontier[0].state);

    let mut depth = 0;

    loop {
        depth += 1;
        let _start_time = Instant::now();
        let frontier_size = frontier.len();

        // Update Status
        {
            let mut s = status.lock().unwrap();
            s.depth = depth;
            s.current_frontier_size = frontier_size;
            if let Some(best) = frontier.first() {
                // Always track the head of the search
                s.best_score = best.score;
                s.best_node = Some(best.clone());
            }
        }

        if frontier.is_empty() {
            break;
        }

        // Check for Victory in current frontier
        for node in &frontier {
            if node.state.phase == GamePhase::Victory {
                info!("Victory Found at depth {}!", depth);
                let mut s = status.lock().unwrap();
                s.finished = true;
                s.best_node = Some(node.clone());
                return;
            }
        }

        info!("Depth {}: Expanding {} nodes...", depth, frontier_size);

        // Parallel Expansion
        let candidates: Vec<StateNode> = frontier
            .par_iter()
            .flat_map(|node| expand_node(node))
            .collect();

        // Scoring & Pruning (Selection)
        // We want the Top K candidates.
        // Convert to Heap or Sort. Sorting is fine for width ~100-1000.
        
        let mut valid_candidates: Vec<StateNode> = candidates
            .into_iter()
            .filter(|n| n.state.phase != GamePhase::GameOver) // Filter dead ends
            .collect();

        // Deduplication (Simple hash of something? Or just exact match?)
        // Exact match of GameState is expensive.
        // For now, skip dedup or use simple heuristic scoring sort.
        
        valid_candidates.sort_by(|a, b| b.score.cmp(&a.score)); // Descending
        valid_candidates.truncate(beam_width);

        frontier = valid_candidates;

        // Update visited count
        {
            let mut s = status.lock().unwrap();
            s.nodes_visited += frontier_size;
        }

        if frontier.is_empty() {
            info!("Search exhausted all paths.");
            let mut s = status.lock().unwrap();
            s.finished = true;
            break;
        }
    }
}

fn expand_node(node: &StateNode) -> Vec<StateNode> {
    let state = &node.state;
    let mut player_ids: Vec<String> = state.players.keys().cloned().collect();
    player_ids.sort();

    // If not in Planning, just advance via VoteReady
    if state.phase != GamePhase::TacticalPlanning {
        let mut next_state = state.clone();
        let mut path_log = node.path.clone();
        for pid in &player_ids {
            let act = Action::VoteReady { ready: true };
            path_log.push((pid.clone(), act.clone()));
            match GameLogic::apply_action(next_state.clone(), pid, act, None) {
                 Ok(s) => next_state = s,
                 Err(_) => return vec![], // Should not happen if game logic is robust
            }
        }
        let score = heuristic::evaluate(&next_state);
        return vec![StateNode {
            state: next_state,
            path: path_log, 
            score,
            depth: node.depth + 1,
        }];
    }

    // 1. Generate Tasks for each player (TacticalPlanning)
    let mut all_tasks: Vec<Vec<PlayerTask>> = Vec::new();
    for pid in &player_ids {
        let tasks = generate_tasks(state, pid);
        if tasks.is_empty() {
            // Must have at least Pass?
            return vec![];
        }
        all_tasks.push(tasks);
    }

    // 2. Cartesian Product of Tasks
    // If we have 3 players with [5, 5, 5] tasks => 125 combinations.
    // We need to iterate them.
    let combinations = cartesian_product(&all_tasks);

    let mut new_nodes = Vec::with_capacity(combinations.len());

    for combo in combinations {
        let mut next_state = state.clone();
        let mut path_log = node.path.clone();

        let mut invalid = false;

        // Apply Actions
        for (i, task) in combo.iter().enumerate() {
            let pid = &player_ids[i];
            
            // Log Macro (Description)
            path_log.push((pid.clone(), Action::Chat { message: format!("[MACRO] {}", task.description) }));

            for act in &task.actions {
                // Log actual action
                path_log.push((pid.clone(), act.clone()));
                // Apply
                match GameLogic::apply_action(next_state.clone(), pid, act.clone(), None) {
                    Ok(s) => next_state = s,
                    Err(_) => {
                        // Invalid move sequence (e.g. blocked path blocked by another player?)
                        // Since we calculate paths based on static map, dynamic conflicts happen.
                        invalid = true; 
                        break;
                    }
                }
            }
            if invalid { break; }
        }

        if invalid { continue; }

        // Vote Ready (All players)
        for pid in &player_ids {
            let vote_act = Action::VoteReady { ready: true };
            path_log.push((pid.clone(), vote_act.clone()));
            match GameLogic::apply_action(next_state.clone(), pid, vote_act, None) {
                 Ok(s) => next_state = s,
                 Err(_) => { invalid = true; break; }
            }
        }

        if invalid { continue; }

        // At this point, `next_state` should have advanced through Execution -> ... -> Planning
        // Or GameOver.
        
        let score = heuristic::evaluate(&next_state);
        
        new_nodes.push(StateNode {
            state: next_state,
            path: path_log,
            score,
            depth: node.depth + 1,
        });
    }

    new_nodes
}

// Helper for Cartesian Product of variable number of vectors
fn cartesian_product<T: Clone>(lists: &[Vec<T>]) -> Vec<Vec<T>> {
    let mut result = vec![vec![]];
    for list in lists {
        let mut next_result = Vec::new();
        for r in result {
            for item in list {
                let mut new_r = r.clone();
                new_r.push(item.clone());
                next_result.push(new_r);
            }
        }
        result = next_result;
    }
    result
}
