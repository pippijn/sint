use super::{actions::action_cost, cards::get_behavior};
use crate::logic::handlers::get_handler;
use crate::types::*;
use log::{debug, info};
use rand::{Rng, SeedableRng, rngs::StdRng};

pub fn resolve_enemy_attack(state: &mut GameState) {
    // 1. Handle Fog Bank (Hidden Attack) or Normal Attack via Hooks
    let mut attack_opt = state.enemy.next_attack.take();

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
            if let Some(room_id) = attack.target_room
                && let Some(room) = state.map.rooms.get_mut(&room_id)
            {
                match attack.effect {
                    AttackEffect::Fireball => {
                        room.add_hazard(HazardType::Fire);
                        for _ in 0..hazard_mod {
                            room.add_hazard(HazardType::Fire);
                        }
                        state.hull_integrity -= 1;
                    }
                    AttackEffect::Leak => {
                        room.add_hazard(HazardType::Water);
                        for _ in 0..hazard_mod {
                            room.add_hazard(HazardType::Water);
                        }
                        state.hull_integrity -= 1;
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

    // Deterministic Iteration: SmallMap keys are returned in order
    let room_ids: smallvec::SmallVec<[u32; 16]> = state.map.rooms.keys().collect();

    // 1. Process Hazards (Fire spreads, Water leaks, System damage)
    for room_id in &room_ids {
        let room = state.map.rooms.get_mut(room_id).unwrap();

        let fire_count = room
            .hazards
            .iter()
            .filter(|&h| *h == HazardType::Fire)
            .count() as u32;

        if fire_count > 0 {
            // Apply damage to system health
            if room.system_health > 0 {
                if room.system_health <= fire_count {
                    room.system_health = 0;
                    room.is_broken = true;
                    // System exploded!
                    state.hull_integrity -= 1;
                    info!("System in {} exploded!", room.name);
                } else {
                    room.system_health -= fire_count;
                }
            }

            // Spread Chance? (Cargo spreads faster: Threshold 1 instead of 2)
            let threshold = if room.system == Some(SystemType::Cargo) {
                1
            } else {
                2
            };

            if fire_count >= threshold {
                for &neighbor in &room.neighbors {
                    if rng.random_bool(0.5) {
                        fire_spreads.push(neighbor);
                    }
                }
            }
        } else {
            // No fire, auto-restore if not broken
            if !room.is_broken && room.system_health < SYSTEM_HEALTH {
                room.system_health = SYSTEM_HEALTH;
            }
        }
    }

    // Apply Player Damage separately
    for p in state.players.values_mut() {
        if let Some(room) = state.map.rooms.get(&p.room_id)
            && room.hazards.contains(&HazardType::Fire)
        {
            p.hp -= 1;
            if p.hp <= 0 {
                p.status.push(PlayerStatus::Fainted);
            }
        }
    }

    // 2. Apply Spreads
    for room_id in fire_spreads {
        if let Some(room) = state.map.rooms.get_mut(&room_id)
            && !room.hazards.contains(&HazardType::Fire)
        {
            room.add_hazard(HazardType::Fire);
        }
    }

    // 3. Water destroys items (Except in Storage)
    for room_id in &room_ids {
        if let Some(room) = state.map.rooms.get_mut(room_id)
            && room.hazards.contains(&HazardType::Water)
            && room.system != Some(SystemType::Storage)
        {
            // Only destroy Peppernuts. Special items survive.
            room.items.retain(|i| *i != ItemType::Peppernut);
        }
    }

    state.rng_seed = rng.random();
}

pub fn process_round_end(state: &mut GameState) {
    // 1. Process Timebombs
    let mut triggered_ids = Vec::new();

    for card in state.active_situations.iter_mut() {
        if let CardType::Timebomb { rounds_left } = &mut card.card_type
            && *rounds_left > 0
        {
            *rounds_left -= 1;
            if *rounds_left == 0 {
                triggered_ids.push(card.id);
            }
        }
    }

    // 2. Call on_trigger for all that just reached 0
    for id in triggered_ids {
        get_behavior(id).on_trigger(state);
    }

    // 3. Trigger Card End-of-Round Effects (Mice, Amerigo, etc.)
    let active_ids: Vec<CardId> = state.active_situations.iter().map(|c| c.id).collect();
    for id in active_ids {
        get_behavior(id).on_round_end(state);
    }
}

pub fn resolve_proposal_queue(state: &mut GameState, simulation: bool) {
    let queue = std::mem::take(&mut state.proposal_queue);

    for proposal in queue {
        let player_id = &proposal.player_id;

        if !state.players.contains_key(player_id) {
            continue;
        }

        // 1. Card Resolution Hook (RNG / Dynamic Blocks)
        let mut blocked_by_card = false;
        let active_ids: Vec<CardId> = state.active_situations.iter().map(|c| c.id).collect();

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

        // 2. Handler Execution
        let handler = get_handler(&proposal.action);
        match handler.execute(state, player_id, simulation) {
            Ok(_) => {
                // Success
            }
            Err(e) => {
                debug!(
                    "Action Skipped: Player {} failed to execute {:?}: {}",
                    player_id, proposal.action, e
                );
                // Refund
                let cost = action_cost(state, player_id, &proposal.action);
                if let Some(p) = state.players.get_mut(player_id) {
                    p.ap += cost;
                }
            }
        }
    }
}
