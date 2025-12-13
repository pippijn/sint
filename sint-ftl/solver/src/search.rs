pub mod beam;
pub mod rhea;

use sint_core::types::{Action, GameAction, GameState, PlayerId};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

pub fn get_legal_actions(state: &GameState) -> Vec<(PlayerId, GameAction)> {
    let players: Vec<_> = state.players.values().collect();
    // Deterministic active player selection
    if let Some(p) = players.into_iter().find(|p| !p.is_ready && p.ap > 0) {
        let legal_wrappers = sint_core::logic::actions::get_valid_actions(state, &p.id);
        let mut valid = Vec::new();
        for w in legal_wrappers {
            if let Action::Game(act) = w {
                // Filter out non-strategic actions like Chat/Undo
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
    pub score: f64,
    pub signature: u64,
}

#[derive(Clone, Debug)]
pub struct SearchProgress {
    pub step: usize,
    pub best_score: f64,
    pub hull: i32,
    pub boss_hp: i32,
    pub is_done: bool,
    pub current_best_node: Option<Arc<SearchNode>>,
}

impl SearchNode {
    pub fn get_history(&self) -> Vec<&(PlayerId, GameAction)> {
        let mut history = Vec::new();
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
