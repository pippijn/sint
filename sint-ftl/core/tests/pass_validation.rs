use sint_core::{Action, GameLogic, GamePhase};

#[test]
fn test_pass_with_ap_works() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // P1 has 2 AP default.
    let p1 = state.players.get("P1").unwrap();
    assert_eq!(p1.ap, 2);

    let res = GameLogic::apply_action(state.clone(), "P1", Action::Pass, None);
    assert!(res.is_ok());
    
    let new_state = res.unwrap();
    let p1_new = new_state.players.get("P1").unwrap();
    assert_eq!(p1_new.ap, 0); // AP should be consumed
    
    // Phase advances to Execution
    assert_eq!(new_state.phase, GamePhase::Execution);
}

#[test]
fn test_pass_with_0_ap_fails() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Set AP to 0
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 0;
    }

    let res = GameLogic::apply_action(state, "P1", Action::Pass, None);
    assert!(res.is_err());
    
    let err = res.err().unwrap();
    assert_eq!(err.to_string(), "Invalid Action: Cannot Pass with 0 AP. Vote ready instead.");
}

#[test]
fn test_vote_ready_works_any_ap() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Case 1: 0 AP
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 0;
    }
    let res = GameLogic::apply_action(state.clone(), "P1", Action::VoteReady { ready: true }, None);
    assert!(res.is_ok());
    let s = res.unwrap();
    assert_eq!(s.phase, GamePhase::Execution);

    // Case 2: 2 AP
    let mut state2 = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state2.phase = GamePhase::TacticalPlanning;
    
    let res2 = GameLogic::apply_action(state2.clone(), "P1", Action::VoteReady { ready: true }, None);
    assert!(res2.is_ok());
    let s2 = res2.unwrap();
    let p1 = s2.players.get("P1").unwrap();
    assert_eq!(p1.ap, 2); // AP Preserved
    assert_eq!(s2.phase, GamePhase::Execution);
}
