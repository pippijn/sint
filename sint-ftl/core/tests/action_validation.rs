use sint_core::{Action, GameLogic, GamePhase};

#[test]
fn test_cannot_evade_from_dormitory() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // P1 starts in Room 3 (Dormitory)
    let p1 = state.players.get("P1").unwrap();
    assert_eq!(p1.room_id, 3);

    // Attempt Evasive Maneuvers (Requires Room 9 Bridge)
    // Should fail
    let res = GameLogic::apply_action(state.clone(), "P1", Action::EvasiveManeuvers, None);
    assert!(
        res.is_err(),
        "Should not allow EvasiveManeuvers from Dormitory"
    );
}

#[test]
fn test_can_evade_from_bridge() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Teleport P1 to Bridge (9)
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sint_core::types::SystemType::Bridge.as_u32();
    }

    let res = GameLogic::apply_action(state, "P1", Action::EvasiveManeuvers, None);
    assert!(res.is_ok());
}

#[test]
fn test_cannot_bake_from_dormitory() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let res = GameLogic::apply_action(state, "P1", Action::Bake, None);
    assert!(res.is_err(), "Should not allow Bake from Dormitory");
}

#[test]
fn test_move_then_evade_valid() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // P1 in Room 3. Move to 7 -> 9 (2 AP)
    // Evasive Maneuvers (2 AP). Total 4 AP. P1 has 2 AP.
    // Give P1 more AP for testing.
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 10;
    }

    // Queue Move to 7
    let state = GameLogic::apply_action(
        state,
        "P1",
        Action::Move {
            to_room: sint_core::types::SystemType::Hallway.as_u32(),
        },
        None,
    )
    .unwrap();
    // Queue Move to 9
    let state = GameLogic::apply_action(
        state,
        "P1",
        Action::Move {
            to_room: sint_core::types::SystemType::Bridge.as_u32(),
        },
        None,
    )
    .unwrap();

    // Now Queue Evasive Maneuvers
    // Projected room should be 9.
    let res = GameLogic::apply_action(state, "P1", Action::EvasiveManeuvers, None);
    assert!(
        res.is_ok(),
        "Should allow EvasiveManeuvers if projected to be in Bridge"
    );
}
