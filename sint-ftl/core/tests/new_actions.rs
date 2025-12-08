use sint_core::{Action, GameLogic, GamePhase};

#[test]
fn test_lookout_action() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Move P1 to Bow (2)
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sint_core::types::SystemType::Bow.as_u32(); // Bow
        p.ap = 2;
    }

    // Action: Lookout
    let res = GameLogic::apply_action(state.clone(), "P1", Action::Lookout, None);
    assert!(res.is_ok(), "Lookout should be valid in Bow");

    // Check Chat Log (Resolution)
    let mut state = res.unwrap();
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

    // Verify Chat Log contains "LOOKOUT REPORT"
    let last_msg = state.chat_log.last().unwrap();
    assert_eq!(last_msg.sender, "SYSTEM");
    assert!(last_msg.text.contains("LOOKOUT REPORT"));
}

#[test]
fn test_first_aid_action() {
    let mut state = GameLogic::new_game(vec!["P1".to_string(), "P2".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Setup: P1 in Sickbay (10), P2 in Hallway (7) [Neighbor]. P2 injured.
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sint_core::types::SystemType::Sickbay.as_u32();
        p.ap = 2;
    }
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = sint_core::types::SystemType::Hallway.as_u32();
        p.hp = 1; // Injured
    }

    // Action: P1 heals P2
    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::FirstAid {
            target_player: "P2".to_string(),
        },
        None,
    );
    assert!(res.is_ok(), "First Aid valid on neighbor");

    // Resolve
    let mut state = res.unwrap();
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    state = GameLogic::apply_action(state, "P2", Action::VoteReady { ready: true }, None).unwrap();

    // Check P2 HP
    assert_eq!(state.players["P2"].hp, 2, "P2 should heal 1 HP");
}

#[test]
fn test_first_aid_invalid_range() {
    let mut state = GameLogic::new_game(vec!["P1".to_string(), "P2".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Setup: P1 in Sickbay (10), P2 in Kitchen (6) [Not Neighbor]
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sint_core::types::SystemType::Sickbay.as_u32();
        p.ap = 2;
    }
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = sint_core::types::SystemType::Kitchen.as_u32();
        p.hp = 1;
    }

    let res = GameLogic::apply_action(
        state,
        "P1",
        Action::FirstAid {
            target_player: "P2".to_string(),
        },
        None,
    );
    assert!(res.is_err(), "First Aid should fail if target too far");
}
