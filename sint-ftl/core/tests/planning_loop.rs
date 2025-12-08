use sint_core::{Action, GameLogic, GamePhase};

#[test]
fn test_planning_loop() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);

    // 1. Start Game -> Morning
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::MorningReport);

    // 2. Ready -> Telegraph
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::EnemyTelegraph);

    // 3. Ready -> Planning
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::TacticalPlanning);
    assert_eq!(state.players["P1"].ap, 2);

    // 4. Move (AP 2 -> 1)
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Move {
            to_room: sint_core::types::SystemType::Hallway.as_u32(),
        },
        None,
    )
    .unwrap();
    assert_eq!(state.players["P1"].ap, 1);

    // 5. Ready -> Execution
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::Execution);

    // 6. Ready (in Execution, with AP 1) -> SHOULD GO BACK TO PLANNING
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::TacticalPlanning);

    // 7. Move again (AP 1 -> 0)
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Move {
            to_room: sint_core::types::SystemType::Kitchen.as_u32(),
        },
        None,
    )
    .unwrap();
    assert_eq!(state.players["P1"].ap, 0);

    // 8. Ready -> Execution
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::Execution);

    // 9. Ready (in Execution, with AP 0) -> SHOULD GO TO ENEMY ACTION
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::EnemyAction);
}
