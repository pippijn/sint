use sint_core::logic::pathfinding;
use sint_core::types::{Action, GameState, HazardType, ItemType, SystemType};

#[derive(Debug, Clone)]
pub struct PlayerTask {
    pub player_id: String,
    pub description: String,
    pub actions: Vec<Action>,
    pub cost: i32,
}

pub fn generate_tasks(state: &GameState, player_id: &str) -> Vec<PlayerTask> {
    let mut tasks = Vec::new();
    let player = state.players.get(player_id).unwrap();
    let start_room = player.room_id;
    let ap = player.ap;

    // 1. Pass (Always valid)
    tasks.push(PlayerTask {
        player_id: player_id.to_string(),
        description: "Pass".to_string(),
        actions: vec![Action::Pass], // Pass clears AP and marks Ready
        cost: 0,
    });

    // Helper to check reachability and cost
    // simplistic: pathfinding doesn't account for dynamic blocks (cards) in this heuristic phase
    // but the execution will fail if blocked, which is fine (invalid branch).

    // 2. Extinguish (Find nearest Fire)
    // Deterministic Room Iteration
    let mut room_ids: Vec<u32> = state.map.rooms.keys().cloned().collect();
    room_ids.sort();

    for room_id in &room_ids {
        let room = &state.map.rooms[room_id];
        if room.hazards.contains(&HazardType::Fire) {
            if let Some(path) = pathfinding::find_path(&state.map, start_room, *room_id) {
                // Cost = Moves + 1 (Extinguish)
                let move_cost = path.len() as i32;
                let total_cost = move_cost + 1;

                if total_cost <= ap {
                    let mut actions = Vec::new();
                    for step in &path {
                        actions.push(Action::Move { to_room: *step });
                    }
                    actions.push(Action::Extinguish);

                    tasks.push(PlayerTask {
                        player_id: player_id.to_string(),
                        description: format!("Extinguish in {}", room.name),
                        actions,
                        cost: total_cost,
                    });
                } else {
                    // Multi-turn: Move as far as possible
                    let mut actions = Vec::new();
                    let mut cost_so_far = 0;
                    for step in &path {
                        if cost_so_far < ap {
                            actions.push(Action::Move { to_room: *step });
                            cost_so_far += 1;
                        } else {
                            break;
                        }
                    }
                    if !actions.is_empty() {
                        tasks.push(PlayerTask {
                            player_id: player_id.to_string(),
                            description: format!("Move towards Fire in {}", room.name),
                            actions,
                            cost: cost_so_far,
                        });
                    }
                }
            }
        }
    }

    // 3. Repair (Find nearest Water)
    for room_id in &room_ids {
        let room = &state.map.rooms[room_id];
        if room.hazards.contains(&HazardType::Water) {
            if let Some(path) = pathfinding::find_path(&state.map, start_room, *room_id) {
                let move_cost = path.len() as i32;
                let total_cost = move_cost + 1;

                if total_cost <= ap {
                    let mut actions = Vec::new();
                    for step in &path {
                        actions.push(Action::Move { to_room: *step });
                    }
                    actions.push(Action::Repair);

                    tasks.push(PlayerTask {
                        player_id: player_id.to_string(),
                        description: format!("Repair in {}", room.name),
                        actions,
                        cost: total_cost,
                    });
                } else {
                    // Multi-turn
                    let mut actions = Vec::new();
                    let mut cost_so_far = 0;
                    for step in &path {
                        if cost_so_far < ap {
                            actions.push(Action::Move { to_room: *step });
                            cost_so_far += 1;
                        } else {
                            break;
                        }
                    }
                    if !actions.is_empty() {
                        tasks.push(PlayerTask {
                            player_id: player_id.to_string(),
                            description: format!("Move towards Water in {}", room.name),
                            actions,
                            cost: cost_so_far,
                        });
                    }
                }
            }
        }
    }

    // 4. Bake (Go to Kitchen)
    let kitchen_id = SystemType::Kitchen.as_u32();
    if let Some(path) = pathfinding::find_path(&state.map, start_room, kitchen_id) {
        let move_cost = path.len() as i32;
        let total_cost = move_cost + 1; // Bake
        if total_cost <= ap {
            let mut actions = Vec::new();
            for step in &path {
                actions.push(Action::Move { to_room: *step });
            }
            actions.push(Action::Bake);
            tasks.push(PlayerTask {
                player_id: player_id.to_string(),
                description: "Bake".to_string(),
                actions,
                cost: total_cost,
            });
        } else {
            // Multi-turn
            let mut actions = Vec::new();
            let mut cost_so_far = 0;
            for step in &path {
                if cost_so_far < ap {
                    actions.push(Action::Move { to_room: *step });
                    cost_so_far += 1;
                } else {
                    break;
                }
            }
            if !actions.is_empty() {
                tasks.push(PlayerTask {
                    player_id: player_id.to_string(),
                    description: "Move towards Kitchen".to_string(),
                    actions,
                    cost: cost_so_far,
                });
            }
        }
    }

    // 5. Shoot (Go to Cannons) - Requires Ammo
    let cannons_id = SystemType::Cannons.as_u32();
    if player.inventory.contains(&ItemType::Peppernut) {
        if let Some(path) = pathfinding::find_path(&state.map, start_room, cannons_id) {
            let move_cost = path.len() as i32;
            let total_cost = move_cost + 1;
            if total_cost <= ap {
                let mut actions = Vec::new();
                for step in &path {
                    actions.push(Action::Move { to_room: *step });
                }
                actions.push(Action::Shoot);
                tasks.push(PlayerTask {
                    player_id: player_id.to_string(),
                    description: "Shoot".to_string(),
                    actions,
                    cost: total_cost,
                });
            } else {
                // Multi-turn
                let mut actions = Vec::new();
                let mut cost_so_far = 0;
                for step in &path {
                    if cost_so_far < ap {
                        actions.push(Action::Move { to_room: *step });
                        cost_so_far += 1;
                    } else {
                        break;
                    }
                }
                if !actions.is_empty() {
                    tasks.push(PlayerTask {
                        player_id: player_id.to_string(),
                        description: "Move towards Cannons".to_string(),
                        actions,
                        cost: cost_so_far,
                    });
                }
            }
        }
    }

    // 6. Pickup Ammo (If in room with ammo)
    // Simplified: Only looks at current room or adjacent.
    // If we look global, the list gets too big.
    // Let's stick to "Current Room" for simple interactions to keep branching factor low.
    if let Some(room) = state.map.rooms.get(&start_room) {
        if room.items.contains(&ItemType::Peppernut)
            && ap >= 1 {
                tasks.push(PlayerTask {
                    player_id: player_id.to_string(),
                    description: "Pickup Ammo".to_string(),
                    actions: vec![Action::PickUp {
                        item_type: ItemType::Peppernut,
                    }],
                    cost: 1,
                });
            }
    }

    // 7. Revive (If fainted player in room)
    if let Some(_room) = state.map.rooms.get(&start_room) {
        let mut player_ids: Vec<String> = state.players.keys().cloned().collect();
        player_ids.sort();
        for pid in &player_ids {
            let other = &state.players[pid];
            if other.room_id == start_room
                && other
                    .status
                    .contains(&sint_core::types::PlayerStatus::Fainted)
                && ap >= 1 {
                    tasks.push(PlayerTask {
                        player_id: player_id.to_string(),
                        description: format!("Revive {}", other.name),
                        actions: vec![Action::Revive {
                            target_player: other.id.clone(),
                        }],
                        cost: 1,
                    });
                }
        }
    }

    // 8. Move to Key Locations (Dorm, Bridge, Engine) just to be there?
    // This is useful for "Positioning".
    // Maybe just generic "Move to Room X" if it's within 1 turn range?
    // Let's add "Move to Bridge" (Shields) and "Move to Engine" (Evasion) if AP permits interaction.

    // Engine (Evasion - Cost 2)
    let engine_id = SystemType::Engine.as_u32();
    if let Some(path) = pathfinding::find_path(&state.map, start_room, engine_id) {
        let move_cost = path.len() as i32;
        let total_cost = move_cost + 2; // Evasion
        if total_cost <= ap {
            let mut actions = Vec::new();
            for step in path {
                actions.push(Action::Move { to_room: step });
            }
            actions.push(Action::EvasiveManeuvers);
            tasks.push(PlayerTask {
                player_id: player_id.to_string(),
                description: "Evasion".to_string(),
                actions,
                cost: total_cost,
            });
        }
    }

    // Limit the number of tasks to avoid explosion?
    // Pass, Extinguish x N, Repair x M, Bake, Shoot...
    // It's probably < 20 per player.

    tasks
}
