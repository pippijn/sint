use sint_core::{
    GameLogic,
    logic::find_room_with_system_in_map,
    types::{Action, GameAction, GamePhase, HazardType, ItemType, MAX_HULL, SystemType},
};

#[test]
fn test_cannot_throw_wheelbarrow() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned(), "P2".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let dormitory =
        sint_core::logic::find_room_with_system_in_map(&state.map, SystemType::Dormitory).unwrap();

    if let Some(p) = state.players.get_mut("P1") {
        p.inventory.push(ItemType::Wheelbarrow); // Index 0
        p.room_id = dormitory;
    }
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = dormitory;
    }

    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Throw {
            target_player: "P2".to_owned(),
            item_index: 0,
        }),
        None,
    );
    assert!(res.is_err(), "Should not be able to throw Wheelbarrow");
}

#[test]
fn test_cannot_throw_extinguisher() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned(), "P2".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let dormitory =
        sint_core::logic::find_room_with_system_in_map(&state.map, SystemType::Dormitory).unwrap();

    if let Some(p) = state.players.get_mut("P1") {
        p.inventory.push(ItemType::Extinguisher); // Index 0
        p.room_id = dormitory;
    }
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = dormitory;
    }

    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Throw {
            target_player: "P2".to_owned(),
            item_index: 0,
        }),
        None,
    );
    assert!(res.is_err(), "Should not be able to throw Extinguisher");
}

#[test]
fn test_cargo_repair_blocked_by_fire() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;
    state.hull_integrity = MAX_HULL - 1;

    let cargo =
        sint_core::logic::find_room_with_system_in_map(&state.map, SystemType::Cargo).unwrap();

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = cargo;
    }
    if let Some(r) = state.map.rooms.get_mut(&cargo) {
        r.hazards.push(HazardType::Fire);
    }

    let res = GameLogic::apply_action(state.clone(), "P1", Action::Game(GameAction::Repair), None);

    // In Cargo, Repair targets Water first, then Hull.
    // If no Water is present, it tries Hull repair.
    // If Fire is present, it should fail.
    assert!(res.is_err(), "Cargo Hull repair should be blocked by Fire");
}

#[test]
fn test_repair_blocked_by_any_hazard() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let cargo = find_room_with_system_in_map(&state.map, SystemType::Cargo).unwrap();

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = cargo;
    }

    // 1. Fire blocks Water repair
    if let Some(r) = state.map.rooms.get_mut(&cargo) {
        r.hazards.push(HazardType::Water);
        r.hazards.push(HazardType::Fire);
    }

    let actions = GameLogic::get_valid_actions(&state, "P1");
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, Action::Game(GameAction::Repair))),
        "Repair should be blocked by Fire"
    );

    // 2. Water blocks Hull repair (Cargo)
    if let Some(r) = state.map.rooms.get_mut(&cargo) {
        r.hazards.clear();
        r.hazards.push(HazardType::Water);
    }
    state.hull_integrity = 10;

    let actions = GameLogic::get_valid_actions(&state, "P1");
    // Repair is available, but it will target Water first.
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, Action::Game(GameAction::Repair)))
    );

    let res = GameLogic::apply_action(state.clone(), "P1", Action::Game(GameAction::Repair), None)
        .unwrap();
    let res = GameLogic::apply_action(
        res,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    // Water should be gone, Hull should still be 10
    assert!(!res.map.rooms[&cargo].hazards.contains(&HazardType::Water));
    assert_eq!(res.hull_integrity, 10);
}
