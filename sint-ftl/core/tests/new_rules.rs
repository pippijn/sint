use sint_core::{
    GameLogic,
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
fn test_cargo_repair_water_allowed_despite_fire() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let cargo =
        sint_core::logic::find_room_with_system_in_map(&state.map, SystemType::Cargo).unwrap();

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = cargo;
    }
    if let Some(r) = state.map.rooms.get_mut(&cargo) {
        r.hazards.push(HazardType::Fire);
        r.hazards.push(HazardType::Water);
    }

    let res = GameLogic::apply_action(state.clone(), "P1", Action::Game(GameAction::Repair), None);

    // Repairing WATER should still work even if Fire is present?
    // Actually, rule says "Disabled: If a room has 1 or more Fire/Water tokens, its Action is unusable."
    // Extinguish and Repair are "Cleanup" actions, not necessarily the "System Action".
    // But hull repair is explicitly an "Action" tied to Cargo.
    // Let's re-read: "Players must Extinguish (Fire) or Repair (Water) to restore function."
    // Cleanup actions should probably be allowed even if disabled, otherwise you can never fix it.

    // Wait, RepairHandler::validate for Water repair doesn't check for Fire.
    // But Cargo Hull Repair DOES check for Fire now.

    assert!(
        res.is_ok(),
        "Should be able to repair Water even if Fire is present"
    );
}
