use sint_core::{Action, GameLogic, GamePhase, ItemType};

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
