use sint_core::{
    logic::{apply_action, GameLogic},
    types::{Action, GameAction, GamePhase},
};

fn get_player_ids(state: &sint_core::types::GameState) -> Vec<String> {
    state.players.keys().cloned().collect()
}

#[test]
fn test_phase_advances_when_all_players_pass() {
    // 1. Setup: Create a game with two players in the TacticalPlanning phase.
    let mut state = GameLogic::new_game(vec!["p1".to_owned(), "p2".to_owned()], 0);
    state.phase = GamePhase::TacticalPlanning;
    for p in state.players.values_mut() {
        p.ap = 2; // Ensure players have AP to pass
    }

    let players = get_player_ids(&state);
    let p1_id = &players[0];
    let p2_id = &players[1];

    // 2. Action: Both players pass, exhausting their AP.
    let pass_action = Action::Game(GameAction::Pass);

    // Player 1 passes. The phase should not advance yet.
    state = apply_action(state, p1_id, pass_action.clone()).unwrap();
    assert_eq!(
        state.phase,
        GamePhase::TacticalPlanning,
        "Phase should not advance while other players still have AP."
    );
    assert_eq!(state.players.get(p1_id).unwrap().ap, 0);
    assert!(state.players.get(p1_id).unwrap().is_ready);

    // Player 2 passes.
    state = apply_action(state, p2_id, pass_action).unwrap();

    // 3. Assert: The phase advances to Execution because all players are done.
    assert_eq!(
        state.phase,
        GamePhase::Execution,
        "Phase should advance to Execution once all players have passed."
    );
}

#[test]
fn test_actions_are_disallowed_outside_planning_phase() {
    // 1. Setup: Create a game with one player.
    let mut state = GameLogic::new_game(vec!["p1".to_owned()], 0);
    let player_id = get_player_ids(&state)[0].clone();

    // Find a valid room to move to
    let start_room = state.players.get(&player_id).unwrap().room_id;
    let target_room = state.map.rooms.get(&start_room).unwrap().neighbors[0];
    let move_action = Action::Game(GameAction::Move {
        to_room: target_room,
    });

    // 2. Action & Assert: Test in various non-planning phases
    let disallowed_phases = [
        GamePhase::Execution,
        GamePhase::EnemyAction,
        GamePhase::MorningReport,
        GamePhase::EnemyTelegraph,
        GamePhase::Lobby,
    ];

    for phase in disallowed_phases {
        state.phase = phase;
        let result = apply_action(state.clone(), &player_id, move_action.clone());
        assert!(
            result.is_err(),
            "Move action should be disallowed during the {:?} phase",
            phase
        );
        let error = result.unwrap_err();
        assert!(matches!(
            error,
            sint_core::logic::GameError::InvalidAction(_)
        ));
    }
}

#[test]
fn test_full_phase_cycle() {
    // 1. Setup: Create a game with one player.
    let mut state = GameLogic::new_game(vec!["p1".to_owned()], 0);
    let player_id = get_player_ids(&state)[0].clone();

    // 2. Action & Assert: Manually step through the phases

    // Start of the game, players ready up in the lobby
    state.phase = GamePhase::Lobby;
    state = apply_action(
        state,
        &player_id,
        Action::Game(GameAction::VoteReady { ready: true }),
    )
    .unwrap();

    // Morning Report (Turn 1)
    assert_eq!(state.phase, GamePhase::MorningReport);
    assert_eq!(state.turn_count, 1);
    assert_eq!(state.players.get(&player_id).unwrap().ap, 2); // AP is reset

    // Players ready up to advance
    state = apply_action(
        state,
        &player_id,
        Action::Game(GameAction::VoteReady { ready: true }),
    )
    .unwrap();
    assert_eq!(state.phase, GamePhase::EnemyTelegraph);
    state = apply_action(
        state,
        &player_id,
        Action::Game(GameAction::VoteReady { ready: true }),
    )
    .unwrap();
    assert_eq!(state.phase, GamePhase::TacticalPlanning);

    // Player takes their turn
    state = apply_action(state, &player_id, Action::Game(GameAction::Pass)).unwrap();
    assert_eq!(state.phase, GamePhase::Execution);

    // After execution, since all AP is used, it should automatically go to EnemyAction
    // Note: The `advance_phase` is called internally. We need to vote ready to trigger the check.
    state = apply_action(
        state,
        &player_id,
        Action::Game(GameAction::VoteReady { ready: true }),
    )
    .unwrap();
    assert_eq!(state.phase, GamePhase::EnemyAction);

    // After EnemyAction, it should loop back to Morning Report for the next turn
    state = apply_action(
        state,
        &player_id,
        Action::Game(GameAction::VoteReady { ready: true }),
    )
    .unwrap();

    // Morning Report (Turn 2)
    assert_eq!(state.phase, GamePhase::MorningReport);
    assert_eq!(state.turn_count, 2, "Turn count should have incremented");
    assert_eq!(
        state.players.get(&player_id).unwrap().ap,
        2,
        "AP should be reset for the new turn"
    );
}
