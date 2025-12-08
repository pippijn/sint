use crate::types::{Action, GameMap, GameState, PlayerId, RoomId};
use std::collections::{HashSet, VecDeque};

pub fn find_path(map: &GameMap, start: RoomId, end: RoomId) -> Option<Vec<RoomId>> {
    if start == end {
        return Some(vec![]);
    }

    let mut queue = VecDeque::new();
    queue.push_back(vec![start]);

    let mut visited = HashSet::new();
    visited.insert(start);

    while let Some(path) = queue.pop_front() {
        let last = *path.last().unwrap();
        if last == end {
            // Return steps excluding start, so [start, next, end] -> [next, end]
            return Some(path.into_iter().skip(1).collect());
        }

        // Limit search depth to avoid infinite loops (though graph is small)
        if path.len() > 10 {
            continue;
        }

        if let Some(room) = map.rooms.get(&last) {
            for &neighbor in &room.neighbors {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    let mut new_path = path.clone();
                    new_path.push(neighbor);
                    queue.push_back(new_path);
                }
            }
        }
    }
    None
}

pub fn get_player_projected_room(state: &GameState, player_id: &PlayerId) -> RoomId {
    let mut current_room = state.players.get(player_id).map(|p| p.room_id).unwrap_or(3); // Default to 3 if not found (shouldn't happen)

    for proposal in &state.proposal_queue {
        if proposal.player_id == *player_id {
            if let Action::Move { to_room } = proposal.action {
                current_room = to_room;
            }
        }
    }
    current_room
}
