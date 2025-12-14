use sint_core::{
    GameLogic,
    logic::{find_room_with_system_in_map, resolution},
    types::*,
};

#[test]
fn test_system_damage_from_fire() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::EnemyAction;

    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    // Initial health should be 3
    assert_eq!(state.map.rooms[&kitchen].system_health, 3);

    // 1 Fire token should deal 1 damage
    if let Some(r) = state.map.rooms.get_mut(&kitchen) {
        r.hazards.push(HazardType::Fire);
    }

    resolution::resolve_hazards(&mut state);

    assert_eq!(state.map.rooms[&kitchen].system_health, 2);
    // Hull should NOT take damage yet
    assert_eq!(state.hull_integrity, 20);
}

#[test]
fn test_system_auto_restore() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::EnemyAction;

    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    // Set system to damaged (2 HP) and NO fire
    if let Some(r) = state.map.rooms.get_mut(&kitchen) {
        r.system_health = 2;
    }

    resolution::resolve_hazards(&mut state);

    // Should auto-restore to 3
    assert_eq!(state.map.rooms[&kitchen].system_health, 3);
}

#[test]
fn test_system_explosion() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::EnemyAction;

    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    // 3 Fire tokens should destroy the system (3 HP)
    if let Some(r) = state.map.rooms.get_mut(&kitchen) {
        r.hazards.push(HazardType::Fire);
        r.hazards.push(HazardType::Fire);
        r.hazards.push(HazardType::Fire);
    }

    resolution::resolve_hazards(&mut state);

    assert_eq!(state.map.rooms[&kitchen].system_health, 0);
    // Hull should take 1 damage from explosion
    assert_eq!(state.hull_integrity, 19);
}

#[test]
fn test_broken_system_functionality() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = kitchen;
    }

    // Damaged system (1 HP) still works
    if let Some(r) = state.map.rooms.get_mut(&kitchen) {
        r.system_health = 1;
    }

    let actions = GameLogic::get_valid_actions(&state, "P1");
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, Action::Game(GameAction::Bake)))
    );

    // Broken system (0 HP) does NOT work
    if let Some(r) = state.map.rooms.get_mut(&kitchen) {
        r.system_health = 0;
        r.is_broken = true;
    }

    let actions = GameLogic::get_valid_actions(&state, "P1");
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, Action::Game(GameAction::Bake)))
    );
}

#[test]
fn test_multiple_fires_more_damage() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::EnemyAction;

    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    // 2 Fire tokens should deal 2 damage
    if let Some(r) = state.map.rooms.get_mut(&kitchen) {
        r.hazards.push(HazardType::Fire);
        r.hazards.push(HazardType::Fire);
    }

    resolution::resolve_hazards(&mut state);

    assert_eq!(state.map.rooms[&kitchen].system_health, 1);
}

#[test]
fn test_broken_until_fully_fixed() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = kitchen;
        p.ap = 5;
    }
    // Broken system
    if let Some(r) = state.map.rooms.get_mut(&kitchen) {
        r.system_health = 0;
        r.is_broken = true;
    }

    // Repair 1 AP -> 1 HP
    state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::Repair), None).unwrap();
    // Resolve manually by calling the function advance_phase would call
    resolution::resolve_proposal_queue(&mut state, false);

    assert_eq!(state.map.rooms[&kitchen].system_health, 1);
    assert!(state.map.rooms[&kitchen].is_broken);

    // System still non-functional at 1 HP
    let actions = GameLogic::get_valid_actions(&state, "P1");
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, Action::Game(GameAction::Bake)))
    );

    // Repair to 3 HP
    state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::Repair), None).unwrap();
    state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::Repair), None).unwrap();
    resolution::resolve_proposal_queue(&mut state, false);

    assert_eq!(state.map.rooms[&kitchen].system_health, 3);
    assert!(!state.map.rooms[&kitchen].is_broken);

    // System now functional
    let actions = GameLogic::get_valid_actions(&state, "P1");
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, Action::Game(GameAction::Bake)))
    );
}

#[test]
fn test_system_repair() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = kitchen;
        p.ap = 3;
    }
    // Broken system
    if let Some(r) = state.map.rooms.get_mut(&kitchen) {
        r.system_health = 0;
    }

    // Repair 1 AP -> 1 HP
    let state =
        GameLogic::apply_action(state, "P1", Action::Game(GameAction::Repair), None).unwrap();
    let state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    assert_eq!(state.map.rooms[&kitchen].system_health, 1);

    // System still broken at 1 HP?
    let actions = GameLogic::get_valid_actions(&state, "P1");
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, Action::Game(GameAction::Bake)))
    );
}

#[test]
fn test_repair_blocked_by_fire() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = kitchen;
    }
    if let Some(r) = state.map.rooms.get_mut(&kitchen) {
        r.system_health = 0;
        r.hazards.push(HazardType::Fire);
    }

    let actions = GameLogic::get_valid_actions(&state, "P1");
    // Repair should NOT be available because of fire
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, Action::Game(GameAction::Repair)))
    );
}

#[test]
fn test_water_blocks_system() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = kitchen;
    }
    // Healthy system but Water present
    if let Some(r) = state.map.rooms.get_mut(&kitchen) {
        r.system_health = 3;
        r.hazards.push(HazardType::Water);
    }

    let actions = GameLogic::get_valid_actions(&state, "P1");
    // System should be blocked by water
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, Action::Game(GameAction::Bake)))
    );
}
