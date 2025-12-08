use sint_core::{Action, GameLogic, GamePhase, HazardType, ItemType};

#[test]
fn test_water_destroys_items() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::Execution;

    // Setup: Kitchen (6) with Water and 2 Nuts.
    if let Some(r) = state.map.rooms.get_mut(&sint_core::types::SystemType::Kitchen.as_u32()) {
        r.hazards.push(HazardType::Water);
        r.items.push(ItemType::Peppernut);
        r.items.push(ItemType::Peppernut);
    }

    // Trigger Resolution
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 0;
        p.is_ready = true;
    }
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

    // Check Kitchen Items (Should be empty)
    if let Some(r) = state.map.rooms.get(&sint_core::types::SystemType::Kitchen.as_u32()) {
        assert!(r.items.is_empty(), "Water should destroy items in Kitchen");
    }
}

#[test]
fn test_storage_protects_items() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::Execution;

    // Setup: Storage (11) with Water and 2 Nuts.
    // (Note: Storage has 5 nuts by default, plus we add 2 more)
    if let Some(r) = state.map.rooms.get_mut(&sint_core::types::SystemType::Storage.as_u32()) {
        r.hazards.push(HazardType::Water);
        r.items.push(ItemType::Peppernut);
        r.items.push(ItemType::Peppernut);
    }

    let initial_count = state.map.rooms[&sint_core::types::SystemType::Storage.as_u32()].items.len();

    // Trigger Resolution
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 0;
        p.is_ready = true;
    }
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

    // Check Storage Items (Should be unchanged)
    if let Some(r) = state.map.rooms.get(&sint_core::types::SystemType::Storage.as_u32()) {
        assert_eq!(
            r.items.len(),
            initial_count,
            "Storage should protect items from Water"
        );
    }
}
