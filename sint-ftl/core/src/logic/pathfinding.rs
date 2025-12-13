use crate::types::{GameMap, RoomId};
use std::collections::{HashSet, VecDeque};

pub struct MapDistances {
    distances: Vec<Vec<u32>>,
}

impl MapDistances {
    pub fn new(map: &GameMap) -> Self {
        let max_id = map.rooms.keys().map(|id| id as usize).max().unwrap_or(0);
        let mut distances = vec![vec![999u32; max_id + 1]; max_id + 1];

        for start_id in map.rooms.keys() {
            let s = start_id as usize;
            distances[s][s] = 0;
            let mut queue = VecDeque::new();
            queue.push_back((start_id, 0));

            while let Some((curr, d)) = queue.pop_front() {
                if let Some(room) = map.rooms.get(&curr) {
                    for &neighbor in &room.neighbors {
                        let n = neighbor as usize;
                        if distances[s][n] == 999 {
                            distances[s][n] = d + 1;
                            queue.push_back((neighbor, d + 1));
                        }
                    }
                }
            }
        }

        Self { distances }
    }

    pub fn get(&self, start: RoomId, end: RoomId) -> u32 {
        self.distances
            .get(start as usize)
            .and_then(|row| row.get(end as usize))
            .copied()
            .unwrap_or(999)
    }

    pub fn min_distance(&self, start: RoomId, targets: &HashSet<RoomId>) -> u32 {
        if targets.is_empty() {
            return 999;
        }
        let mut min_d = 999;
        for &target in targets {
            let d = self.get(start, target);
            if d < min_d {
                min_d = d;
                if min_d == 0 {
                    break;
                }
            }
        }
        min_d
    }
}

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
