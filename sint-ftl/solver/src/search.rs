use crate::scoring::ScoreDetails;
use sint_core::types::{GameAction, GameState, PlayerId};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

pub mod beam;
pub mod config;
pub mod rhea;

pub fn get_valid_actions(state: &GameState) -> Vec<(PlayerId, GameAction)> {
    let mut valid = Vec::new();
    // Deterministic active player selection: First player who is not ready and has AP
    if let Some(p) = state.players.values().find(|p| !p.is_ready && p.ap > 0) {
        let actions = sint_core::logic::actions::get_valid_actions(state, &p.id);
        for action in actions {
            if let sint_core::types::Action::Game(act) = action {
                if matches!(
                    act,
                    GameAction::Chat { .. }
                        | GameAction::Undo { .. }
                        | GameAction::VoteReady { .. }
                ) {
                    continue;
                }
                valid.push((p.id.clone(), act));
            }
        }
        return valid;
    }
    Vec::new()
}

#[derive(Clone, Debug)]
pub struct SearchNode {
    pub state: GameState,
    pub parent: Option<Arc<SearchNode>>,
    pub last_action: Option<(PlayerId, GameAction)>,
    pub score: ScoreDetails,
    pub signature: u64,
    pub history_len: usize,
}

#[derive(Clone, Debug)]
pub struct SearchProgress {
    pub step: usize,
    pub is_done: bool,
    pub failed: bool,
    pub node: Arc<SearchNode>,
}

impl SearchNode {
    pub fn get_history(&self) -> Vec<&(PlayerId, GameAction)> {
        let mut history = Vec::with_capacity(self.history_len);
        let mut current = self;
        while let Some(parent) = &current.parent {
            if let Some(action) = &current.last_action {
                history.push(action);
            }
            current = parent;
        }
        history.reverse();
        history
    }

    pub fn get_recent_history(&self, n: usize) -> Vec<&(PlayerId, GameAction)> {
        let mut history = Vec::with_capacity(n);
        let mut current = self;
        while let Some(parent) = &current.parent {
            if history.len() >= n {
                break;
            }
            if let Some(action) = &current.last_action {
                history.push(action);
            }
            current = parent;
        }
        history.reverse();
        history
    }
}

impl PartialEq for SearchNode {
    fn eq(&self, other: &Self) -> bool {
        self.score.total == other.score.total
    }
}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for SearchNode {}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}
impl std::hash::Hash for SearchNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ((self.score.total * 1000.0) as i64).hash(state);
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

pub fn get_state_signature(state: &GameState) -> u64 {
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
