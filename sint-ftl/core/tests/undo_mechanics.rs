use sint_core::{Action, GameLogic, GamePhase};

#[test]
fn test_undo_middle_of_chain() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // P1 starts in Room 3 (Dormitory). Neighbors include 7 (Hallway).
    // P1 has 2 AP.

    // 1. Queue Move 3 -> 7 (Cost 1)
    state = GameLogic::apply_action(state, "P1", Action::Move { to_room: 7 }, None).unwrap();
    assert_eq!(state.players["P1"].ap, 1);
    let id_move_7 = state.proposal_queue[0].id.clone();

    // 2. Queue Move 7 -> 9 (Cost 1) (Room 9 is Bridge, neighbor of 7)
    state = GameLogic::apply_action(state, "P1", Action::Move { to_room: 9 }, None).unwrap();
    assert_eq!(state.players["P1"].ap, 0);
    assert_eq!(state.proposal_queue.len(), 2);

    // 3. Undo the FIRST action (Move 3 -> 7)
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Undo {
            action_id: id_move_7,
        },
        None,
    )
    .unwrap();

    // Verification:
    // a. Queue length should be 1
    assert_eq!(state.proposal_queue.len(), 1);
    // b. Remaining action should be Move 7 -> 9
    if let Action::Move { to_room } = &state.proposal_queue[0].action {
        assert_eq!(*to_room, 9);
    } else {
        panic!("Remaining action should be Move to 9");
    }
    // c. AP should be refunded for the first move (0 -> 1)
    assert_eq!(state.players["P1"].ap, 1);

    // 4. Execute Batch
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::Execution);

    // Execution Logic runs here implicitly when entering Execution phase.
    // The resolution has happened.
    // P1 should have AP refunded (1 + 1 = 2)
    assert_eq!(state.players["P1"].ap, 2);
    // Check Room
    assert_eq!(state.players["P1"].room_id, 3);

    // 5. Acknowledge Execution (Transition Execution -> TacticalPlanning)
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

    // Phase should loop back to Planning because AP > 0
    assert_eq!(state.phase, GamePhase::TacticalPlanning);
}
