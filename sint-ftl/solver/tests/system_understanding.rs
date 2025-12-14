use sint_core::logic::{GameLogic, apply_action};
use sint_core::types::*;
use sint_solver::driver::GameDriver;

#[test]
fn test_ap_reset_on_round_transition() {
    let player_ids = vec!["P1".to_owned(), "P2".to_owned()];
    let seed = 2236;
    let mut state = GameLogic::new_game(player_ids.clone(), seed);
    
    // Stabilize to TacticalPlanning
    let mut driver = GameDriver::new(state);
    assert_eq!(driver.state.phase, GamePhase::TacticalPlanning);
    
    let turn = driver.state.turn_count;
    
    // Everyone passes
    for pid in &player_ids {
        driver.apply(pid, GameAction::Pass).unwrap();
    }
    
    // Should be next round, stabilized back to TacticalPlanning
    assert_eq!(driver.state.turn_count, turn + 1);
    assert_eq!(driver.state.phase, GamePhase::TacticalPlanning);
    
    // Check AP
    for pid in &player_ids {
        let p = driver.state.players.get(pid).unwrap();
        assert_eq!(p.ap, 2, "Player {} should have 2 AP at start of round {}", pid, driver.state.turn_count);
    }
}

#[test]
fn test_ap_reset_after_boss_defeat() {
    let player_ids = vec!["P1".to_owned()];
    let seed = 2236;
    let mut state = GameLogic::new_game(player_ids.clone(), seed);
    
    // Teleport P1 to Cannons and give ammo
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 6; // Cannons
        p.inventory.push(ItemType::Peppernut);
        p.inventory.push(ItemType::Peppernut);
        p.inventory.push(ItemType::Peppernut);
        p.inventory.push(ItemType::Peppernut);
        p.inventory.push(ItemType::Peppernut);
    }
    
    // Set boss HP to 1
    state.enemy.hp = 1;
    
    let mut driver = GameDriver::new(state);
    
    // Shoot boss
    // We need a hit. ShootHandler uses RNG. 
    // For testing we might want to bypass RNG or just try until hit if we don't want to mock.
    // But since we can't easily mock here without changing code, let's just observe.
    // In execute_solution, P1 shoots and it hits.
    
    // To ensure a hit in test without changing core, we can loop or set seed.
    // Actually, ShootHandler in simulation mode doesn't apply damage.
    // But GameDriver::apply calls it with simulation=false.
    
    let mut hit = false;
    for _ in 0..10 {
        if driver.state.enemy.hp <= 0 {
            hit = true;
            break;
        }
        if driver.state.players["P1"].ap == 0 {
            // End round to get more AP
            driver.apply("P1", GameAction::Pass).unwrap();
        }
        let _ = driver.apply("P1", GameAction::Shoot);
    }
    
    assert!(driver.state.enemy.hp <= 0 || driver.state.enemy.state == EnemyState::Defeated);
    
    // If boss defeated, we should be in/approaching rest round
    // Stabilize handles transitions.
    
    if driver.state.is_resting {
        assert_eq!(driver.state.players["P1"].ap, 6);
    } else {
        // Round transition should have reset AP to 2 if not resting
        assert_eq!(driver.state.players["P1"].ap, 2);
    }
}

#[test]
fn test_faint_and_respawn_ap() {
    let player_ids = vec!["P1".to_owned()];
    let seed = 2236;
    let mut state = GameLogic::new_game(player_ids.clone(), seed);
    
    // Put P1 in a room with fire
    state.map.rooms.get_mut(&0).unwrap().hazards.push(HazardType::Fire);
    state.players.get_mut("P1").unwrap().room_id = 0;
    state.players.get_mut("P1").unwrap().hp = 1;
    
    let mut driver = GameDriver::new(state);
    
    // Pass to trigger end of round hazards
    driver.apply("P1", GameAction::Pass).unwrap();
    
    // P1 should have fainted and respawned
    let p = &driver.state.players["P1"];
    assert_eq!(p.hp, 3);
    assert_eq!(p.room_id, 2); // Dormitory
    assert_eq!(p.ap, 2);
}
