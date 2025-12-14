use sint_core::{
    logic::{GameError, GameLogic, apply_action},
    types::*,
};

#[test]
fn test_multiple_shield_actions_fail() {
    // 1. Setup: Create a game with 2 players in the Bridge.
    let mut state = GameLogic::new_game(vec!["p1".to_owned(), "p2".to_owned()], 0);
    state.phase = GamePhase::TacticalPlanning;

    let bridge_id =
        sint_core::logic::find_room_with_system_in_map(&state.map, SystemType::Bridge).unwrap();

    for p in state.players.values_mut() {
        p.room_id = bridge_id;
    }

    // 2. Action: P1 raises shields.
    let shield_action = Action::Game(GameAction::RaiseShields);
    let state = apply_action(state, "p1", shield_action.clone())
        .expect("P1 should be able to raise shields");

    // 3. Action: P2 attempts to raise shields.
    let result = apply_action(state, "p2", shield_action);

    // 4. Assert: The second action should fail.
    assert!(
        result.is_err(),
        "P2 should NOT be able to raise shields if they are already being raised (projected state)"
    );
    let err = result.unwrap_err();
    // We haven't defined the error yet, but let's assume it will be InvalidAction for now.
    assert!(
        matches!(err, GameError::InvalidAction(_)),
        "Expected InvalidAction, got {:?}",
        err
    );
}

#[test]
fn test_get_valid_actions_filters_redundant_shield_and_evasion() {
    // 1. Setup: Create a game with 1 player in the Bridge.
    let mut state = GameLogic::new_game(vec!["p1".to_owned()], 0);
    state.phase = GamePhase::TacticalPlanning;

    let bridge_id =
        sint_core::logic::find_room_with_system_in_map(&state.map, SystemType::Bridge).unwrap();
    let engine_id =
        sint_core::logic::find_room_with_system_in_map(&state.map, SystemType::Engine).unwrap();

    let p = state.players.get_mut("p1").unwrap();
    p.room_id = bridge_id;

    // 2. Action: Verify RaiseShields is initially valid.
    let valid = GameLogic::get_valid_actions(&state, "p1");
    assert!(
        valid
            .iter()
            .any(|a| matches!(a, Action::Game(GameAction::RaiseShields)))
    );

    // 3. Action: P1 raises shields.
    let mut state = apply_action(state, "p1", Action::Game(GameAction::RaiseShields)).unwrap();

    // 4. Verify RaiseShields is no longer valid.
    let valid = GameLogic::get_valid_actions(&state, "p1");
    assert!(
        !valid
            .iter()
            .any(|a| matches!(a, Action::Game(GameAction::RaiseShields)))
    );

    // 5. Setup: Move player to Engine.
    let p = state.players.get_mut("p1").unwrap();
    p.room_id = engine_id;
    p.ap = 10; // Give some AP

    // 6. Verify EvasiveManeuvers is valid.
    let valid = GameLogic::get_valid_actions(&state, "p1");
    assert!(
        valid
            .iter()
            .any(|a| matches!(a, Action::Game(GameAction::EvasiveManeuvers)))
    );

    // 7. Action: P1 engages evasive maneuvers.
    let state = apply_action(state, "p1", Action::Game(GameAction::EvasiveManeuvers)).unwrap();

    // 8. Verify EvasiveManeuvers is no longer valid.
    let valid = GameLogic::get_valid_actions(&state, "p1");
    assert!(
        !valid
            .iter()
            .any(|a| matches!(a, Action::Game(GameAction::EvasiveManeuvers)))
    );
}
