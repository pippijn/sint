use sint_core::ItemType;
use sint_core::{Action, GameLogic, GamePhase, HazardType};

#[test]
fn test_fire_damage() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);

    // Setup: P1 in Kitchen (6), Fire in Kitchen
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sint_core::types::SystemType::Kitchen.as_u32();
        p.ap = 0; // Force end of turn
        p.is_ready = true;
    }

    if let Some(room) = state
        .map
        .rooms
        .get_mut(&sint_core::types::SystemType::Kitchen.as_u32())
    {
        room.hazards.push(HazardType::Fire);
    }

    let initial_hull = state.hull_integrity;
    let initial_hp = state.players["P1"].hp;

    // Transition: Execution -> EnemyAction (Triggers resolve_hazards)
    state.phase = GamePhase::Execution;

    // We need to trigger "VoteReady" or "Pass" to advance phase, but since AP is 0,
    // apply_action(VoteReady) should trigger advance_phase to EnemyAction.
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

    assert_eq!(state.phase, GamePhase::EnemyAction);

    // Check Damage
    assert_eq!(
        state.hull_integrity,
        initial_hull - 1,
        "Hull should take fire damage"
    );
    assert_eq!(
        state.players["P1"].hp,
        initial_hp - 1,
        "Player should take fire damage"
    );
}

#[test]
fn test_extinguish_fire() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Setup: P1 in Kitchen (6), Fire in Kitchen
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sint_core::types::SystemType::Kitchen.as_u32();
        p.ap = 2;
    }
    if let Some(room) = state
        .map
        .rooms
        .get_mut(&sint_core::types::SystemType::Kitchen.as_u32())
    {
        room.hazards.push(HazardType::Fire);
    }

    // Action: Extinguish
    state = GameLogic::apply_action(state, "P1", Action::Extinguish, None).unwrap();

    // Check AP cost
    assert_eq!(state.players["P1"].ap, 1);

    // Advance to Execution to resolve
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::Execution);

    // Trigger Resolution (VoteReady again or implicit if auto-trigger? apply_action doesn't auto-resolve logic unless phase advanced)
    // Wait, advance_phase(TacticalPlanning) -> calls resolve_proposal_queue immediately!
    // So Extinguish should be done.

    if let Some(room) = state
        .map
        .rooms
        .get(&sint_core::types::SystemType::Kitchen.as_u32())
    {
        assert!(room.hazards.is_empty(), "Fire should be extinguished");
    }
}

#[test]
fn test_repair_water() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Setup: Water in Kitchen
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sint_core::types::SystemType::Kitchen.as_u32();
    }
    if let Some(room) = state
        .map
        .rooms
        .get_mut(&sint_core::types::SystemType::Kitchen.as_u32())
    {
        room.hazards.push(HazardType::Water);
    }

    // Action: Repair
    state = GameLogic::apply_action(state, "P1", Action::Repair, None).unwrap();

    // Advance to resolve
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

    if let Some(room) = state
        .map
        .rooms
        .get(&sint_core::types::SystemType::Kitchen.as_u32())
    {
        assert!(room.hazards.is_empty(), "Water should be repaired");
    }
}

#[test]
fn test_kitchen_standard_threshold() {
    // Kitchen should NOT spread with 1 Fire.
    for i in 0..20 {
        let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345 + i);
        state.phase = GamePhase::Execution;
        if let Some(r) = state
            .map
            .rooms
            .get_mut(&sint_core::types::SystemType::Kitchen.as_u32())
        {
            r.hazards.push(HazardType::Fire);
        }
        if let Some(r) = state
            .map
            .rooms
            .get_mut(&sint_core::types::SystemType::Hallway.as_u32())
        {
            r.hazards.clear();
        }
        if let Some(p) = state.players.get_mut("P1") {
            p.ap = 0;
            p.is_ready = true;
        }
        state =
            GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

        if let Some(r) = state
            .map
            .rooms
            .get(&sint_core::types::SystemType::Hallway.as_u32())
        {
            assert!(
                r.hazards.is_empty(),
                "Standard room with 1 Fire should NOT spread"
            );
        }
    }
}

#[test]
fn test_cargo_lower_threshold() {
    // Cargo SHOULD spread with 1 Fire (approx 50% of time).
    let mut spread_occured = false;

    for i in 0..20 {
        let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345 + i);
        state.phase = GamePhase::Execution;
        if let Some(r) = state
            .map
            .rooms
            .get_mut(&sint_core::types::SystemType::Cargo.as_u32())
        {
            r.hazards.push(HazardType::Fire);
        }
        if let Some(r) = state
            .map
            .rooms
            .get_mut(&sint_core::types::SystemType::Hallway.as_u32())
        {
            r.hazards.clear();
        }
        if let Some(p) = state.players.get_mut("P1") {
            p.ap = 0;
            p.is_ready = true;
        }
        state =
            GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

        if let Some(r) = state
            .map
            .rooms
            .get(&sint_core::types::SystemType::Hallway.as_u32())
        {
            if r.hazards.contains(&HazardType::Fire) {
                spread_occured = true;
                break;
            }
        }
    }
    assert!(spread_occured, "Cargo with 1 Fire SHOULD spread eventually");
}

#[test]
fn test_water_destroys_items() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::Execution;

    // Setup: Kitchen (6) with Water and 2 Nuts.
    if let Some(r) = state
        .map
        .rooms
        .get_mut(&sint_core::types::SystemType::Kitchen.as_u32())
    {
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
    if let Some(r) = state
        .map
        .rooms
        .get(&sint_core::types::SystemType::Kitchen.as_u32())
    {
        assert!(r.items.is_empty(), "Water should destroy items in Kitchen");
    }
}

#[test]
fn test_storage_protects_items() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::Execution;

    // Setup: Storage (11) with Water and 2 Nuts.
    // (Note: Storage has 5 nuts by default, plus we add 2 more)
    if let Some(r) = state
        .map
        .rooms
        .get_mut(&sint_core::types::SystemType::Storage.as_u32())
    {
        r.hazards.push(HazardType::Water);
        r.items.push(ItemType::Peppernut);
        r.items.push(ItemType::Peppernut);
    }

    let initial_count = state.map.rooms[&sint_core::types::SystemType::Storage.as_u32()]
        .items
        .len();

    // Trigger Resolution
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 0;
        p.is_ready = true;
    }
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

    // Check Storage Items (Should be unchanged)
    if let Some(r) = state
        .map
        .rooms
        .get(&sint_core::types::SystemType::Storage.as_u32())
    {
        assert_eq!(
            r.items.len(),
            initial_count,
            "Storage should protect items from Water"
        );
    }
}

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
        assert!(
            !r.items.contains(&ItemType::Peppernut),
            "Water should destroy Peppernuts"
        );
        assert!(
            r.items.contains(&ItemType::Wheelbarrow),
            "Water should NOT destroy Wheelbarrow"
        );
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

    let initial_count = state.map.rooms[&sint_core::types::SystemType::Storage.as_u32()]
        .items
        .len();

    // Trigger Resolution
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 0;
        p.is_ready = true;
    }
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

    // Check Storage
    let r = &state.map.rooms[&sint_core::types::SystemType::Storage.as_u32()];
    assert_eq!(
        r.items.len(),
        initial_count,
        "Storage should protect everything"
    );
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
    if let Some(r) = state
        .map
        .rooms
        .get_mut(&sint_core::types::SystemType::Kitchen.as_u32())
    {
        r.hazards.push(HazardType::Water);
    }

    // Attempt Bake
    let res = GameLogic::apply_action(state, "P1", Action::Bake, None);
    assert!(res.is_err(), "Water should disable Bake action");
}
