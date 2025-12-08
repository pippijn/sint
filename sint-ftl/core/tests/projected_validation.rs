use sint_core::logic::actions::get_valid_actions;
use sint_core::{Action, GameLogic, GamePhase, ItemType};

#[test]
fn test_projected_location_validation() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // P1 starts in Room 3 (Dormitory). Neighbors: [7].
    // Give P1 extra AP for testing chaining.
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 10;
    }

    // 1. Initial State: Valid moves should be [7]
    let actions = get_valid_actions(&state, "P1");
    let has_move_7 = actions
        .iter()
        .any(|a| matches!(a, Action::Move { to_room: 7 }));
    assert!(has_move_7, "Should be able to move to Room 7");

    // 2. Queue Move to 7
    state = GameLogic::apply_action(state, "P1", Action::Move { to_room: 7 }, None).unwrap();

    // 3. Projected State: Should be in Room 7. Neighbors: [2,3,4,5,6,8,9,10,11]
    let actions = get_valid_actions(&state, "P1");

    // Should NOT be able to move to 7 (already there in projection)
    // Actually Move logic checks neighbors. 7 is not neighbor of 7.
    let has_move_7 = actions
        .iter()
        .any(|a| matches!(a, Action::Move { to_room: 7 }));
    assert!(
        !has_move_7,
        "Should NOT be able to move to Room 7 (already there)"
    );

    // Should be able to move to 6 (Kitchen)
    let has_move_6 = actions
        .iter()
        .any(|a| matches!(a, Action::Move { to_room: 6 }));
    assert!(has_move_6, "Should be able to move to Room 6 (Kitchen)");

    // Should NOT be able to Bake (requires being in Kitchen)
    let has_bake = actions.iter().any(|a| matches!(a, Action::Bake));
    assert!(!has_bake, "Should NOT be able to Bake from Hallway");
}

#[test]
fn test_projected_system_availability() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 10;
        p.room_id = 7; // Start in Hallway
    }

    // Queue Move to Kitchen (6)
    state = GameLogic::apply_action(state, "P1", Action::Move { to_room: 6 }, None).unwrap();

    // Projected: In Kitchen.
    let actions = get_valid_actions(&state, "P1");
    let has_bake = actions.iter().any(|a| matches!(a, Action::Bake));
    assert!(has_bake, "Should be able to Bake after moving to Kitchen");
}

#[test]
fn test_projected_ap_exhaustion() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // P1 has 2 AP.
    // Queue Move (1 AP).
    state = GameLogic::apply_action(state, "P1", Action::Move { to_room: 7 }, None).unwrap();

    // Remaining AP: 1.
    let actions = get_valid_actions(&state, "P1");
    let has_move = actions.iter().any(|a| matches!(a, Action::Move { .. }));
    assert!(has_move, "Should still be able to move with 1 AP");

    // Queue another Move (1 AP).
    // From 7 (Hallway) to 6 (Kitchen).
    state = GameLogic::apply_action(state, "P1", Action::Move { to_room: 6 }, None).unwrap();

    // Remaining AP: 0.
    let actions = get_valid_actions(&state, "P1");
    let has_move = actions.iter().any(|a| matches!(a, Action::Move { .. }));
    let has_bake = actions.iter().any(|a| matches!(a, Action::Bake));

    assert!(!has_move, "Should NOT be able to move with 0 AP");
    assert!(!has_bake, "Should NOT be able to bake with 0 AP");

    // Free actions should still be valid
    let has_chat = actions.iter().any(|a| matches!(a, Action::Chat { .. }));
    assert!(has_chat, "Should be able to chat with 0 AP");
}

#[test]
fn test_projected_item_pickup() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Setup: P1 in Room 3. Room 3 has items? Default setup Room 3 has NO items.
    // Let's add an item to Room 3.
    if let Some(r) = state.map.rooms.get_mut(&3) {
        r.items.push(ItemType::Peppernut);
    }

    // Initial check
    let actions = get_valid_actions(&state, "P1");
    let has_pickup = actions.iter().any(|a| {
        matches!(
            a,
            Action::PickUp {
                item_type: ItemType::Peppernut
            }
        )
    });
    assert!(has_pickup, "Should be able to pickup item");

    // Queue Pickup
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::PickUp {
            item_type: ItemType::Peppernut,
        },
        None,
    )
    .unwrap();

    // Projected: Item is in inventory, not room.
    let actions = get_valid_actions(&state, "P1");
    let has_pickup = actions.iter().any(|a| matches!(a, Action::PickUp { .. }));
    assert!(
        !has_pickup,
        "Should NOT be able to pickup item (already picked up in projection)"
    );
}
