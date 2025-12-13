use crate::types::{GameMap, RoomId};
use std::collections::VecDeque;

pub fn find_path(map: &GameMap, start: RoomId, end: RoomId) -> Option<Vec<RoomId>> {
    if start == end {
        return Some(vec![]);
    }

    let mut queue = VecDeque::new();
    queue.push_back(start);

    let mut parents = std::collections::HashMap::new();
    parents.insert(start, None);

    while let Some(current) = queue.pop_front() {
        if current == end {
            let mut path = Vec::new();
            let mut curr = end;
            while let Some(&Some(prev)) = parents.get(&curr) {
                path.push(curr);
                curr = prev;
            }
            path.reverse();
            return Some(path);
        }

        if let Some(room) = map.rooms.get(&current) {
            for &neighbor in &room.neighbors {
                if let std::collections::hash_map::Entry::Vacant(e) = parents.entry(neighbor) {
                    e.insert(Some(current));
                    queue.push_back(neighbor);
                }
            }
        }
    }
    None
}
