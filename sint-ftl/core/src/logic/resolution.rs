use crate::types::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub fn resolve_enemy_attack(state: &mut GameState) {
    if let Some(attack) = &state.enemy.next_attack {
        // Check Evasion
        if state.evasion_active {
            // Attack Misses!
            println!("Attack Missed due to Evasive Maneuvers!");
            state.enemy.next_attack = None;
            return;
        }

        // Check Shields (Blocks Damage Only)
        // For simplicity: Shields block ALL effects for now, or just hull damage?
        // Rules say: "Blocks the next incoming Damage event."
        // We'll treat it as blocking the hit entirely for this version.
        if state.shields_active {
            println!("Shields Blocked the Attack!");
            state.enemy.next_attack = None;
            return;
        }

        // Hit!
        if let Some(room) = state.map.rooms.get_mut(&attack.target_room) {
            match attack.effect {
                AttackEffect::Fireball => {
                    room.hazards.push(HazardType::Fire);
                    state.hull_integrity -= 1; // Direct hit damage?
                }
                AttackEffect::Leak => {
                    room.hazards.push(HazardType::Water);
                }
                _ => {}
            }
        }
    }
    state.enemy.next_attack = None;
}

pub fn resolve_hazards(state: &mut GameState) {
    let mut fire_spreads = vec![];
    let mut rng = StdRng::seed_from_u64(state.rng_seed);

    // 1. Damage Players & Hull
    for room in state.map.rooms.values() {
        let has_fire = room.hazards.contains(&HazardType::Fire);

        if has_fire {
            state.hull_integrity -= 1;

            // Spread Chance? (Simple: if > 1 fire, spread to neighbors)
            if room
                .hazards
                .iter()
                .filter(|&h| *h == HazardType::Fire)
                .count()
                >= 2
            {
                for &neighbor in &room.neighbors {
                    if rng.gen_bool(0.5) {
                        // 50% chance to spread
                        fire_spreads.push(neighbor);
                    }
                }
            }
        }
    }

    // Apply Player Damage separately to avoid borrow issues
    for p in state.players.values_mut() {
        if let Some(room) = state.map.rooms.get(&p.room_id) {
            if room.hazards.contains(&HazardType::Fire) {
                p.hp -= 1;
                if p.hp <= 0 {
                    p.status.push(PlayerStatus::Fainted);
                }
            }
        }
    }

    // 2. Apply Spreads
    for room_id in fire_spreads {
        if let Some(room) = state.map.rooms.get_mut(&room_id) {
            if !room.hazards.contains(&HazardType::Fire) {
                room.hazards.push(HazardType::Fire);
            }
        }
    }

    state.rng_seed = rng.gen();
}

pub fn resolve_proposal_queue(state: &mut GameState) {
    // We clone the queue to iterate while mutating state
    let queue = state.proposal_queue.clone();
    state.proposal_queue.clear();

    for proposal in queue {
        let player_id = &proposal.player_id;

        // Ensure player exists (sanity check)
        if !state.players.contains_key(player_id) {
            continue;
        }

        match proposal.action {
            Action::Move { to_room } => {
                let p = state.players.get(player_id).unwrap();
                let current_room_id = p.room_id;

                // VALIDATION:
                if let Some(room) = state.map.rooms.get(&current_room_id) {
                    if !room.neighbors.contains(&to_room) {
                        // Invalid Move: Skip
                        println!(
                            "Action Skipped: Player {} cannot move from {} to {}",
                            player_id, current_room_id, to_room
                        );
                        continue;
                    }
                }

                // C03: Seagull Attack
                let seagull = state.active_situations.iter().any(|c| c.id == "C03");
                if seagull && p.inventory.contains(&ItemType::Peppernut) {
                    println!(
                        "Action Skipped: Player {} cannot move with Peppernut (Seagulls)",
                        player_id
                    );
                    continue;
                }

                if let Some(p) = state.players.get_mut(player_id) {
                    p.room_id = to_room;
                }
            }
            Action::RaiseShields => {
                let p = state.players.get_mut(player_id).unwrap();
                let room_id = p.room_id;

                if let Some(room) = state.map.rooms.get(&room_id) {
                    // Room 5 (Engine) now handles Shields
                    if room.system == Some(SystemType::Engine) {
                        state.shields_active = true;
                    }
                }
            }
            Action::EvasiveManeuvers => {
                let p = state.players.get_mut(player_id).unwrap();
                let room_id = p.room_id;

                if let Some(room) = state.map.rooms.get(&room_id) {
                    // Room 9 (Bridge) now handles Evasion
                    if room.system == Some(SystemType::Bridge) {
                        state.evasion_active = true;
                    }
                }
            }
            Action::Interact => {
                // Complex logic: depends on room state which might have changed?
                // We'll re-run the logic to find what to solve.
                let p_copy = state.players.get(player_id).cloned().unwrap(); // Copy for read
                let mut solved_idx = None;

                for (i, card) in state.active_situations.iter().enumerate() {
                    if let Some(sol) = &card.solution {
                        if let Some(req_room) = sol.room_id {
                            if req_room != p_copy.room_id {
                                continue;
                            }
                        }
                        if let Some(req_item) = &sol.item_cost {
                            if !p_copy.inventory.contains(req_item) {
                                continue;
                            }
                        }
                        solved_idx = Some(i);
                        break;
                    }
                }

                if let Some(idx) = solved_idx {
                    let card = &state.active_situations[idx];
                    // Pay Cost (Item)
                    if let Some(sol) = &card.solution {
                        if let Some(req_item) = &sol.item_cost {
                            if let Some(p) = state.players.get_mut(player_id) {
                                if let Some(pos) = p.inventory.iter().position(|x| x == req_item) {
                                    p.inventory.remove(pos);
                                }
                            }
                        }
                    }
                    state.active_situations.remove(idx);
                }
            }
            Action::Extinguish => {
                let room_id = state.players.get(player_id).unwrap().room_id;
                if let Some(room) = state.map.rooms.get_mut(&room_id) {
                    if let Some(idx) = room.hazards.iter().position(|&h| h == HazardType::Fire) {
                        room.hazards.remove(idx);
                    }
                }
            }
            Action::Bake => {
                let room_id = state.players.get(player_id).unwrap().room_id;

                // Validation
                if let Some(room) = state.map.rooms.get(&room_id) {
                    if room.system != Some(SystemType::Kitchen) {
                        continue;
                    }
                    if !room.hazards.is_empty() {
                        continue;
                    }
                }

                if let Some(room) = state.map.rooms.get_mut(&room_id) {
                    room.items.push(ItemType::Peppernut);
                    room.items.push(ItemType::Peppernut);
                    room.items.push(ItemType::Peppernut);
                }
            }
            Action::Shoot => {
                let p = state.players.get_mut(player_id).unwrap();
                let room_id = p.room_id;

                // Validation
                if let Some(room) = state.map.rooms.get(&room_id) {
                    if room.system != Some(SystemType::Cannons) {
                        continue;
                    }
                }

                if let Some(idx) = p.inventory.iter().position(|i| *i == ItemType::Peppernut) {
                    p.inventory.remove(idx);

                    // Roll
                    let mut rng = StdRng::seed_from_u64(state.rng_seed);
                    let roll: u32 = rng.gen_range(1..=6);
                    state.rng_seed = rng.gen();

                    if roll >= 3 {
                        state.enemy.hp -= 1;
                    }
                }
            }
            Action::PickUp { item_type } => {
                let room_id = state.players.get(player_id).unwrap().room_id;
                if let Some(room) = state.map.rooms.get_mut(&room_id) {
                    if let Some(pos) = room.items.iter().position(|x| *x == item_type) {
                        let item = room.items.remove(pos);
                        if let Some(p) = state.players.get_mut(player_id) {
                            p.inventory.push(item);
                        }
                    } else {
                        println!(
                            "Action Skipped: Player {} cannot pick up {:?} (Not in room)",
                            player_id, item_type
                        );
                    }
                }
            }
            Action::Throw {
                target_player,
                item_index,
            } => {
                // Removed from source, added to target
                let mut item = None;
                if let Some(p) = state.players.get_mut(player_id) {
                    if item_index < p.inventory.len() {
                        item = Some(p.inventory.remove(item_index));
                    }
                }
                if let Some(it) = item {
                    if let Some(target) = state.players.get_mut(&target_player) {
                        target.inventory.push(it);
                    }
                }
            }
            _ => {} // Other actions might be stubs or instant
        }
    }
}
