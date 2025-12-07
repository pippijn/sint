use crate::types::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub fn resolve_enemy_attack(state: &mut GameState) {
    if let Some(attack) = &state.enemy.next_attack {
        // Check Evasion (TODO: Add Evasion Flag to State)
        // Check Shields (TODO: Add Shield Flag to State)
        
        // Hit!
        if let Some(room) = state.map.rooms.get_mut(&attack.target_room) {
             match attack.effect {
                 AttackEffect::Fireball => {
                     room.hazards.push(HazardType::Fire);
                     state.hull_integrity -= 1; // Direct hit damage?
                 },
                 AttackEffect::Leak => {
                     room.hazards.push(HazardType::Water);
                 },
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
            if room.hazards.iter().filter(|&h| *h == HazardType::Fire).count() >= 2 {
                for &neighbor in &room.neighbors {
                    if rng.gen_bool(0.5) { // 50% chance to spread
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
