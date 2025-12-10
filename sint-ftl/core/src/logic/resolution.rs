use super::{actions::action_cost, cards::get_behavior};
use crate::types::*;
use log::{debug, info};
use rand::{rngs::StdRng, Rng, SeedableRng};

pub fn resolve_enemy_attack(state: &mut GameState) {
    // 1. Handle Fog Bank (Hidden Attack) or Normal Attack via Hooks
    let mut attack_opt = state.enemy.next_attack.clone();

    if let Some(ref mut attack) = attack_opt {
        // Collect IDs to avoid borrow issues while mutating state in hook
        let active_ids: Vec<CardId> = state.active_situations.iter().map(|c| c.id).collect();
        for id in active_ids {
            get_behavior(id).resolve_telegraph(state, attack);
        }
    }

    if let Some(attack) = &attack_opt {
        // Calculate Attack Count
        let mut count = 1;
        let mut hazard_mod = 0;

        for card in &state.active_situations {
            let behavior = get_behavior(card.id);
            let c = behavior.get_enemy_attack_count(state);
            if c > count {
                count = c;
            }
            hazard_mod += behavior.get_hazard_modifier(state);
        }

        for _ in 0..count {
            // Check Evasion
            if state.evasion_active {
                info!("Attack Missed due to Evasive Maneuvers!");
                continue;
            }

            // Check Shields
            if state.shields_active {
                info!("Shields Blocked the Attack!");
                continue;
            }

            // Hit!
            if let Some(room) = state.map.rooms.get_mut(&attack.target_room) {
                match attack.effect {
                    AttackEffect::Fireball => {
                        room.hazards.push(HazardType::Fire);
                        for _ in 0..hazard_mod {
                            room.hazards.push(HazardType::Fire);
                        }
                        state.hull_integrity -= 1;
                    }
                    AttackEffect::Leak => {
                        room.hazards.push(HazardType::Water);
                        for _ in 0..hazard_mod {
                            room.hazards.push(HazardType::Water);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    state.enemy.next_attack = None;
}

pub fn resolve_hazards(state: &mut GameState) {
    let mut fire_spreads = vec![];
    let mut rng = StdRng::seed_from_u64(state.rng_seed);

    // Deterministic Iteration: Sort Room IDs
    let mut room_ids: Vec<u32> = state.map.rooms.keys().cloned().collect();
    room_ids.sort();

    // 1. Damage Players & Hull
    for room_id in &room_ids {
        let room = &state.map.rooms[room_id];
        let has_fire = room.hazards.contains(&HazardType::Fire);

        if has_fire {
            state.hull_integrity -= 1;

            // Spread Chance? (Cargo spreads faster: Threshold 1 instead of 2)
            let threshold = if room.system == Some(SystemType::Cargo) {
                1
            } else {
                2
            };

            if room
                .hazards
                .iter()
                .filter(|&h| *h == HazardType::Fire)
                .count()
                >= threshold
            {
                for &neighbor in &room.neighbors {
                    if rng.gen_bool(0.5) {
                        fire_spreads.push(neighbor);
                    }
                }
            }
        }
    }

    // Apply Player Damage separately
    // Deterministic Iteration: Sort Player IDs
    let mut player_ids: Vec<String> = state.players.keys().cloned().collect();
    player_ids.sort();

    for pid in &player_ids {
        if let Some(p) = state.players.get_mut(pid) {
            if let Some(room) = state.map.rooms.get(&p.room_id) {
                if room.hazards.contains(&HazardType::Fire) {
                    p.hp -= 1;
                    if p.hp <= 0 {
                        p.status.push(PlayerStatus::Fainted);
                    }
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

    // 3. Water destroys items (Except in Storage)
    for room_id in &room_ids {
        if let Some(room) = state.map.rooms.get_mut(room_id) {
            if room.hazards.contains(&HazardType::Water) && room.system != Some(SystemType::Storage)
            {
                // Only destroy Peppernuts. Special items survive.
                room.items.retain(|i| *i != ItemType::Peppernut);
            }
        }
    }

    state.rng_seed = rng.gen();
}

pub fn resolve_proposal_queue(state: &mut GameState, simulation: bool) {
    let queue = state.proposal_queue.clone();
    state.proposal_queue.clear();

    for proposal in queue {
        let player_id = &proposal.player_id;

        if !state.players.contains_key(player_id) {
            continue;
        }

        // 1. Card Resolution Hook (RNG / Dynamic Blocks)
        let mut blocked_by_card = false;
        let active_ids: Vec<CardId> = state.active_situations.iter().map(|c| c.id).collect();

        // In simulation, we assume cards do not block via RNG to avoid oracle behavior.
        // We only check if simulation is FALSE or if the card check is deterministic?
        // Sticky Floor is RNG. If we check it in simulation, we reveal the roll.
        // So in simulation, we SKIP the check_resolution hook?
        // Or we pass 'simulation' to check_resolution?
        // Changing trait `check_resolution` is invasive.
        // For now, let's skip RNG hooks in simulation.

        if !simulation {
            for card_id in active_ids {
                if let Err(e) =
                    get_behavior(card_id).check_resolution(state, player_id, &proposal.action)
                {
                    debug!("Action Skipped: Blocked by card {:?}: {}", card_id, e);
                    blocked_by_card = true;
                    break;
                }
            }
        }

        if blocked_by_card {
            // Refund Logic (Generic)
            let cost = action_cost(state, player_id, &proposal.action);
            if let Some(p) = state.players.get_mut(player_id) {
                p.ap += cost;
            }
            continue;
        }

        match &proposal.action {
            GameAction::Move { to_room } => {
                let p = state.players.get(player_id).unwrap();
                let current_room_id = p.room_id;

                // VALIDATION:
                if let Some(room) = state.map.rooms.get(&current_room_id) {
                    if !room.neighbors.contains(to_room) {
                        debug!(
                            "Action Skipped: Player {} cannot move from {} to {}",
                            player_id, current_room_id, to_room
                        );

                        let cost = action_cost(state, player_id, &proposal.action);
                        if let Some(p) = state.players.get_mut(player_id) {
                            p.ap += cost;
                        }
                        continue;
                    }
                }

                if let Some(p) = state.players.get_mut(player_id) {
                    p.room_id = *to_room;
                }
            }
            GameAction::RaiseShields => {
                let p = state.players.get_mut(player_id).unwrap();
                let room_id = p.room_id;

                if let Some(room) = state.map.rooms.get(&room_id) {
                    if room.system == Some(SystemType::Engine) {
                        state.shields_active = true;
                    }
                }
            }
            GameAction::EvasiveManeuvers => {
                let p = state.players.get_mut(player_id).unwrap();
                let room_id = p.room_id;

                if let Some(room) = state.map.rooms.get(&room_id) {
                    if room.system == Some(SystemType::Bridge) {
                        state.evasion_active = true;
                    }
                }
            }
            GameAction::Interact => {
                let p_copy = state.players.get(player_id).cloned().unwrap();
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

                    // Trigger Reward Hook
                    get_behavior(card.id).on_solved(state);

                    if let Some(sol) = &state.active_situations[idx].solution {
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
            GameAction::Extinguish => {
                let p = state.players.get(player_id).unwrap();
                let has_extinguisher = p.inventory.contains(&ItemType::Extinguisher);
                let room_id = p.room_id;

                if let Some(room) = state.map.rooms.get_mut(&room_id) {
                    let limit = if has_extinguisher { 2 } else { 1 };
                    let mut removed = 0;

                    while removed < limit {
                        if let Some(idx) = room.hazards.iter().position(|&h| h == HazardType::Fire)
                        {
                            room.hazards.remove(idx);
                            removed += 1;
                        } else {
                            break;
                        }
                    }
                }
            }
            GameAction::Repair => {
                let room_id = state.players.get(player_id).unwrap().room_id;
                if let Some(room) = state.map.rooms.get_mut(&room_id) {
                    if let Some(idx) = room.hazards.iter().position(|&h| h == HazardType::Water) {
                        room.hazards.remove(idx);
                    }
                }
            }
            GameAction::Bake => {
                let room_id = state.players.get(player_id).unwrap().room_id;
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
            GameAction::Shoot => {
                let p = state.players.get_mut(player_id).unwrap();
                let room_id = p.room_id;
                if let Some(room) = state.map.rooms.get(&room_id) {
                    if room.system != Some(SystemType::Cannons) {
                        continue;
                    }
                }

                if let Some(idx) = p.inventory.iter().position(|i| *i == ItemType::Peppernut) {
                    p.inventory.remove(idx);

                    // MASK RNG IN SIMULATION
                    if !simulation {
                        let mut rng = StdRng::seed_from_u64(state.rng_seed);
                        let roll: u32 = rng.gen_range(1..=6);
                        state.rng_seed = rng.gen();

                        let mut threshold = 3;
                        for card in &state.active_situations {
                            let t = get_behavior(card.id).get_hit_threshold(state);
                            if t > threshold {
                                threshold = t;
                            }
                        }

                        if roll >= threshold {
                            state.enemy.hp -= 1;

                            // Check for Boss Death
                            if state.enemy.hp <= 0 {
                                state.boss_level += 1;
                                if state.boss_level >= crate::logic::MAX_BOSS_LEVEL {
                                    state.phase = GamePhase::Victory;
                                } else {
                                    // Spawn next boss
                                    state.enemy = crate::logic::get_boss(state.boss_level);

                                    // Announce
                                    state.chat_log.push(ChatMessage {
                                        sender: "SYSTEM".to_string(),
                                        text: format!(
                                            "Enemy Defeated! approaching: {}",
                                            state.enemy.name
                                        ),
                                        timestamp: 0,
                                    });
                                }
                            }
                        }
                    } else {
                        // In simulation, we consume the ammo but don't roll dice or damage enemy.
                        // We also don't advance state.rng_seed to avoid "using" the roll.
                        // Wait, if we DON'T advance rng_seed, then the next simulation step starts with the SAME seed.
                        // That is correct for simulation (exploring).
                        // If we execute, we use the seed.
                    }
                }
            }
            GameAction::Lookout => {
                let card = state.deck.last();
                let msg = if let Some(c) = card {
                    format!(
                        "LOOKOUT REPORT: The next event is '{}' ({})",
                        c.title, c.description
                    )
                } else {
                    "LOOKOUT REPORT: The horizon is clear (Deck Empty).".to_string()
                };

                state.chat_log.push(ChatMessage {
                    sender: "SYSTEM".to_string(),
                    text: msg,
                    timestamp: 0,
                });
            }
            GameAction::FirstAid { target_player } => {
                if let Some(target) = state.players.get_mut(target_player) {
                    if target.hp < 3 {
                        target.hp += 1;
                    }
                }
            }
            GameAction::PickUp { item_type } => {
                let room_id = state.players.get(player_id).unwrap().room_id;
                if let Some(room) = state.map.rooms.get_mut(&room_id) {
                    if let Some(pos) = room.items.iter().position(|x| x == item_type) {
                        let item = room.items.remove(pos);
                        if let Some(p) = state.players.get_mut(player_id) {
                            p.inventory.push(item);
                        }
                    } else {
                        debug!(
                            "Action Skipped: Player {} cannot pick up {:?} (Not in room)",
                            player_id, item_type
                        );
                        let cost = action_cost(state, player_id, &proposal.action);
                        if let Some(p) = state.players.get_mut(player_id) {
                            p.ap += cost;
                        }
                    }
                }
            }
            GameAction::Throw {
                target_player,
                item_index,
            } => {
                let mut item = None;
                if let Some(p) = state.players.get_mut(player_id) {
                    if *item_index < p.inventory.len() {
                        item = Some(p.inventory.remove(*item_index));
                    }
                }
                if let Some(it) = item {
                    if let Some(target) = state.players.get_mut(target_player) {
                        target.inventory.push(it);
                    }
                }
            }
            GameAction::Drop { item_index } => {
                let mut item = None;
                let mut room_id = 0;
                if let Some(p) = state.players.get_mut(player_id) {
                    room_id = p.room_id;
                    if *item_index < p.inventory.len() {
                        item = Some(p.inventory.remove(*item_index));
                    }
                }
                if let Some(it) = item {
                    if let Some(room) = state.map.rooms.get_mut(&room_id) {
                        room.items.push(it);
                    }
                }
            }
            GameAction::Revive { target_player } => {
                // Check if target is in same room and Fainted
                let mut valid = false;
                let room_id = state.players.get(player_id).unwrap().room_id;

                if let Some(target) = state.players.get(target_player) {
                    if target.room_id == room_id && target.status.contains(&PlayerStatus::Fainted) {
                        valid = true;
                    }
                }

                if valid {
                    if let Some(target) = state.players.get_mut(target_player) {
                        target.status.retain(|s| *s != PlayerStatus::Fainted);
                        target.hp = 1;
                    }
                }
            }
            _ => {}
        }
    }
}
