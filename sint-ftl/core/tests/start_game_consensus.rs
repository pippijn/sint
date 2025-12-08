use sint_core::{Action, GameLogic, GamePhase};

#[test]
fn test_single_player_start() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    assert_eq!(state.phase, GamePhase::Lobby);

    // P1 votes ready
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

    // Should start immediately
    assert_eq!(state.phase, GamePhase::MorningReport);
}

#[test]
fn test_multi_player_consensus() {
    let mut state = GameLogic::new_game(vec!["P1".to_string(), "P2".to_string()], 12345);

    // P1 votes ready
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::Lobby); // Not yet

    // P2 votes ready
    state = GameLogic::apply_action(state, "P2", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::MorningReport); // Now
}

#[test]
fn test_multi_player_join_late() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);

    // P1 votes ready... wait, if P1 votes ready, it starts immediately if only 1 player.
    // So we need P1 to NOT be ready, then P2 joins.

    // 1. P1 joins (is_ready=false)
    assert_eq!(state.phase, GamePhase::Lobby);

    // 2. P2 joins
    let join_action = Action::Join {
        name: "P2".to_string(),
    };
    state = GameLogic::apply_action(state, "P2", join_action, None).unwrap();

    assert!(state.players.contains_key("P2"));
    assert_eq!(state.phase, GamePhase::Lobby);

    // 3. P1 votes ready
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::Lobby); // P2 not ready

    // 4. P2 votes ready
    state = GameLogic::apply_action(state, "P2", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::MorningReport);
}
