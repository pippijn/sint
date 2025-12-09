use sint_core::types::{Action, GameState};
use std::cmp::Ordering;

#[derive(Clone)]
pub struct StateNode {
    pub state: GameState,
    pub path: Vec<(String, Action)>, // (PlayerID, Action)
    pub score: i32,
    pub depth: usize,
}

impl StateNode {
    pub fn new(state: GameState) -> Self {
        StateNode {
            state,
            path: Vec::new(),
            score: 0,
            depth: 0,
        }
    }
}

// Implement Eq/Ord for MinHeap/Sorting (Reverse for MaxHeap behavior usually, but we sort descending)
impl PartialEq for StateNode {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for StateNode {}

impl PartialOrd for StateNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StateNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}
