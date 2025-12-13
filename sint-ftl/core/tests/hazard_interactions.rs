use sint_core::{
    GameLogic,
    logic::{find_room_with_system_in_map, resolution},
    types::*,
};

#[test]
fn test_fire_disables_system() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    // Place P1 in Kitchen
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = kitchen;
    }

    // Add Fire
    if let Some(r) = state.map.rooms.get_mut(&kitchen) {
        r.hazards.push(HazardType::Fire);
    }

    // Try to Bake (Should fail)
    let res = GameLogic::apply_action(state.clone(), "P1", Action::Game(GameAction::Bake), None);
    assert!(res.is_err(), "System actions should be disabled by Fire");
}

#[test]
fn test_water_disables_system() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let bridge = find_room_with_system_in_map(&state.map, SystemType::Bridge).unwrap();

    // Place P1 in Bridge
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = bridge;
    }

    // Add Water
    if let Some(r) = state.map.rooms.get_mut(&bridge) {
        r.hazards.push(HazardType::Water);
    }

    // Try Evasive Maneuvers (Should fail)
    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::EvasiveManeuvers),
        None,
    );
    assert!(res.is_err(), "System actions should be disabled by Water");
}

#[test]
fn test_hazard_coexistence() {
    // This tests if Fire and Water can exist in the same room without instantly canceling out
    // (unless resolution logic specifically handles it, which usually happens in EnemyAction phase)
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    if let Some(r) = state.map.rooms.get_mut(&kitchen) {
        r.hazards.push(HazardType::Fire);
        r.hazards.push(HazardType::Water);
    }

    // Check state persistence before resolution
    let r = state.map.rooms.get(&kitchen).unwrap();
    assert!(r.hazards.contains(&HazardType::Fire));
    assert!(r.hazards.contains(&HazardType::Water));

    // Run resolution (EnemyAction phase)
    state.phase = GamePhase::EnemyAction;
    resolution::resolve_hazards(&mut state);

    // Current logic: Hazards resolve damage/spread.
    // They generally don't extinguish each other in `resolve_hazards` unless explicitly coded.
    // If no interaction code exists, both should remain (or spread).
    let r = state.map.rooms.get(&kitchen).unwrap();
    // We verify they persist or resolve independently.
    assert!(r.hazards.contains(&HazardType::Fire), "Fire should persist");
    assert!(
        r.hazards.contains(&HazardType::Water),
        "Water should persist"
    );
}
