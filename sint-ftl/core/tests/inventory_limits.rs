use sint_core::{Action, GameError, GameLogic, GamePhase, ItemType};

#[test]
fn test_inventory_limit_basic() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Setup: P1 in Storage (11). 5 Nuts on floor.
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 11;
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
    if let Some(r) = state.map.rooms.get_mut(&11) {
        r.items.push(ItemType::Peppernut);
        r.items.push(ItemType::Peppernut);
        r.items.push(ItemType::Peppernut);
    }
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 11;
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
    if let Some(r) = state.map.rooms.get_mut(&11) {
        r.items.push(ItemType::Extinguisher);
    }
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 11;
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
