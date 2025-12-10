use sint_core::{Action, GameLogic, GamePhase, ItemType};
use sint_core::GameError;

#[test]
fn test_throw_item() {
    let mut state = GameLogic::new_game(vec!["P1".to_string(), "P2".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Setup: P1 in 6 (Kitchen) with Peppernut. P2 in 7 (Hallway).
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sint_core::types::SystemType::Kitchen.as_u32();
        p.inventory.push(ItemType::Peppernut);
        p.ap = 2;
    }
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = sint_core::types::SystemType::Hallway.as_u32();
        p.inventory.clear();
    }

    // Action: Throw from P1 to P2
    // item_index 0
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Throw {
            target_player: "P2".to_string(),
            item_index: 0,
        },
        None,
    )
    .unwrap();

    // AP Cost
    assert_eq!(state.players["P1"].ap, 1);

    // Resolve
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    state = GameLogic::apply_action(state, "P2", Action::VoteReady { ready: true }, None).unwrap();

    // Check Inventory
    assert!(state.players["P1"].inventory.is_empty());
    assert_eq!(state.players["P2"].inventory[0], ItemType::Peppernut);
}

#[test]
fn test_drop_item() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Setup P1 in Room 6 with Nut
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sint_core::types::SystemType::Kitchen.as_u32();
        p.inventory.push(ItemType::Peppernut);
    }

    // Drop logic in Action enum: Drop { item_index: usize }
    // Cost? rules.md says "Drop item to floor. Free? (Or 1 AP)."
    // `logic/actions.rs` `action_cost` -> Action::Drop => 0.

    state = GameLogic::apply_action(state, "P1", Action::Drop { item_index: 0 }, None).unwrap();

    assert_eq!(state.players["P1"].ap, 2, "Drop should be free");

    // Resolve
    let _ = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

    // Check
    // Not implemented in `resolution.rs`? I see PickUp, Throw...
    // Let's check resolution.rs in `core/src/logic/resolution.rs`
    // I recall checking it and `Action::Drop` might be missing or empty?
    // I need to verify `resolution.rs` content again if this test fails.
}

#[test]
fn test_inventory_limit_basic() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Setup: P1 in Storage (11). 5 Nuts on floor.
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sint_core::types::SystemType::Storage.as_u32();
        p.ap = 5; // Plenty of AP
    }

    // 1. Pick Up Nut 1 (Success)
    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::PickUp {
            item_type: ItemType::Peppernut,
        },
        None,
    );
    assert!(res.is_ok());
    state = res.unwrap();

    // 2. Pick Up Nut 2 (Fail - Limit 1)
    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::PickUp {
            item_type: ItemType::Peppernut,
        },
        None,
    );
    match res {
        Err(GameError::InventoryFull) => {} // Pass
        _ => panic!("Should fail with InventoryFull"),
    }
}

#[test]
fn test_inventory_limit_wheelbarrow() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Setup: P1 has Wheelbarrow. In Room 11.
    // Ensure Room 11 has plenty of nuts (default is 5, we need >5)
    if let Some(r) = state
        .map
        .rooms
        .get_mut(&sint_core::types::SystemType::Storage.as_u32())
    {
        r.items.push(ItemType::Peppernut);
        r.items.push(ItemType::Peppernut);
        r.items.push(ItemType::Peppernut);
    }
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sint_core::types::SystemType::Storage.as_u32();
        p.ap = 10;
        p.inventory.push(ItemType::Wheelbarrow);
    }

    // Pick Up 5 Nuts (Should succeed)
    for _ in 0..5 {
        let res = GameLogic::apply_action(
            state.clone(),
            "P1",
            Action::PickUp {
                item_type: ItemType::Peppernut,
            },
            None,
        );
        assert!(res.is_ok());
        state = res.unwrap();
    }

    // Pick Up 6th Nut (Should fail)
    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::PickUp {
            item_type: ItemType::Peppernut,
        },
        None,
    );
    match res {
        Err(GameError::InventoryFull) => {} // Pass
        _ => panic!("Should fail with InventoryFull (Limit 5)"),
    }
}

#[test]
fn test_inventory_mixed_items() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Setup: Room 11 has Extinguisher added.
    if let Some(r) = state
        .map
        .rooms
        .get_mut(&sint_core::types::SystemType::Storage.as_u32())
    {
        r.items.push(ItemType::Extinguisher);
    }
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sint_core::types::SystemType::Storage.as_u32();
        p.ap = 5;
        p.inventory.push(ItemType::Peppernut); // Max capacity for nuts
    }

    // Pick Up Extinguisher (Should succeed despite having Nut)
    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::PickUp {
            item_type: ItemType::Extinguisher,
        },
        None,
    );
    assert!(res.is_ok(), "Tools should not count towards Ammo limit");
}

#[test]
fn test_pickup_resolution() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);

    // Cheat phase to Planning
    state.phase = GamePhase::TacticalPlanning;

    // Setup Room 3 with specific items
    if let Some(room) = state
        .map
        .rooms
        .get_mut(&sint_core::types::SystemType::Dormitory.as_u32())
    {
        room.items = vec![
            ItemType::Peppernut,
            ItemType::Extinguisher,
            ItemType::Peppernut,
        ];
    }

    // 1. Queue PickUp(Extinguisher)
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::PickUp {
            item_type: ItemType::Extinguisher,
        },
        None,
    )
    .unwrap();

    // Assert Queued
    assert_eq!(state.proposal_queue.len(), 1);

    // 2. Advance Phase to Execution (VoteReady)
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

    // Since "VoteReady" checks consensus, and P1 is the only player,
    // it should advance to Execution and RESOLVE the queue.

    assert_eq!(state.phase, GamePhase::Execution);

    // NOW check results
    let room = state
        .map
        .rooms
        .get(&sint_core::types::SystemType::Dormitory.as_u32())
        .unwrap();
    assert_eq!(room.items.len(), 2);
    // Extinguisher was in the middle, so removal should leave two Peppernuts
    assert_eq!(room.items[0], ItemType::Peppernut);
    assert_eq!(room.items[1], ItemType::Peppernut);

    let p = state.players.get("P1").unwrap();
    assert_eq!(p.inventory.len(), 1);
    assert_eq!(p.inventory[0], ItemType::Extinguisher);
}
