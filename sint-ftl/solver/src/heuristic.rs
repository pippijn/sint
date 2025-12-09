use sint_core::logic::pathfinding;
use sint_core::types::{GamePhase, GameState, HazardType, ItemType, PlayerStatus, SystemType};

pub fn evaluate(state: &GameState) -> i32 {
    let mut score = 0;

    // 1. Progress (The Goal)
    score += (state.boss_level as i32) * 5000;

    // Urgency: Penalize time taken
    score -= (state.turn_count as i32) * 50;

    if state.phase == GamePhase::Victory {
        score += 100_000;
    }

    if state.phase == GamePhase::GameOver {
        return -100_000;
    }

    // 2. Enemy Damage
    // The current enemy HP. Lower is better.
    // Max HP is roughly 20.
    score += (state.enemy.max_hp - state.enemy.hp) * 500;

    // Penalty for remaining HP (Urgency)
    score -= state.enemy.hp * 100;

    // 3. Hull Integrity (Survival)
    score += state.hull_integrity * 100;

    // 4. Hazards (Bad)
    let mut fire_count = 0;
    let mut water_count = 0;
    for room in state.map.rooms.values() {
        for h in &room.hazards {
            match h {
                HazardType::Fire => fire_count += 1,
                HazardType::Water => water_count += 1,
            }
        }
    }
    score -= fire_count * 40; // Fire is bad, but maybe tradeable for a kill
    score -= water_count * 10; // Water is annoying

    // 5. Crew Status
    let mut fainted_count = 0;
    for p in state.players.values() {
        if p.status.contains(&PlayerStatus::Fainted) {
            fainted_count += 1;
        }
        // Health Bonus
        score += p.hp * 10;

        // Inventory Bonus (Resources are good)
        for item in &p.inventory {
            match item {
                ItemType::Peppernut => {
                    score += 20; // Ammo is valuable
                                 // Shaping: Reward proximity to Cannons (Room 8)
                    let cannon_id = SystemType::Cannons.as_u32();
                    if p.room_id == cannon_id {
                        score += 50; // At the cannon!
                    } else if let Some(path) =
                        pathfinding::find_path(&state.map, p.room_id, cannon_id)
                    {
                        // Closer is better. Path len is distance.
                        // Max distance is ~5.
                        score += (10 - path.len() as i32) * 5;
                    }
                }
                ItemType::Extinguisher => score += 10,
                _ => score += 1,
            }
        }
    }
    score -= fainted_count * 200;

    // 6. Bad Situations (Cards)
    score -= (state.active_situations.len() as i32) * 20;

    score
}
