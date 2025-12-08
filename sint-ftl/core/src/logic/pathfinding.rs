use crate::types::{GameMap, RoomId};
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
