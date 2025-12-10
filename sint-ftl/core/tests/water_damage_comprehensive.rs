use sint_core::{Action, GameLogic, GamePhase, HazardType, ItemType};

#[test]
fn test_water_destroys_peppernuts_only() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::Execution;

    // Setup: Kitchen (6) with Water, 2 Nuts, and 1 Wheelbarrow.
    if let Some(r) = state
        .map
        .rooms
        .get_mut(&sint_core::types::SystemType::Kitchen.as_u32())
    {
        r.hazards.push(HazardType::Water);
        r.items.push(ItemType::Peppernut);
        r.items.push(ItemType::Wheelbarrow);
        r.items.push(ItemType::Peppernut);
    }

    // Trigger Resolution
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 0;
        p.is_ready = true;
    }
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

    // Check Kitchen Items
    if let Some(r) = state
        .map
        .rooms
        .get(&sint_core::types::SystemType::Kitchen.as_u32())
    {
        assert!(!r.items.contains(&ItemType::Peppernut), "Water should destroy Peppernuts");
        assert!(r.items.contains(&ItemType::Wheelbarrow), "Water should NOT destroy Wheelbarrow");
        assert_eq!(r.items.len(), 1, "Only Wheelbarrow should remain");
    }
}

#[test]
fn test_storage_protects_all_items() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::Execution;

    // Setup: Storage (11) with Water, Peppernuts, and Extinguisher.
    if let Some(r) = state
        .map
        .rooms
        .get_mut(&sint_core::types::SystemType::Storage.as_u32())
    {
        r.hazards.push(HazardType::Water);
        // Default has 5 nuts. Add Extinguisher.
        r.items.push(ItemType::Extinguisher);
    }

    let initial_count = state.map.rooms[&sint_core::types::SystemType::Storage.as_u32()].items.len();

    // Trigger Resolution
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 0;
        p.is_ready = true;
    }
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

    // Check Storage
    let r = &state.map.rooms[&sint_core::types::SystemType::Storage.as_u32()];
    assert_eq!(r.items.len(), initial_count, "Storage should protect everything");
    assert!(r.items.contains(&ItemType::Peppernut));
    assert!(r.items.contains(&ItemType::Extinguisher));
}

#[test]
fn test_water_disables_room_action() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // P1 in Kitchen with Water
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sint_core::types::SystemType::Kitchen.as_u32();
    }
    if let Some(r) = state.map.rooms.get_mut(&sint_core::types::SystemType::Kitchen.as_u32()) {
        r.hazards.push(HazardType::Water);
    }

    // Attempt Bake
    let res = GameLogic::apply_action(state, "P1", Action::Bake, None);
    assert!(res.is_err(), "Water should disable Bake action");
}
