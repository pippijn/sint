use sint_core::{Action, GameLogic, GamePhase};
use sint_core::GameError;
use sint_core::logic::actions::get_valid_actions;
use sint_core::ItemType;

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

#[test]
fn test_move_in_lobby() {
    let state = GameLogic::new_game(vec!["Player1".to_string()], 12345);

    assert_eq!(state.phase, GamePhase::Lobby);
    let player = state.players.get("Player1").unwrap();
    assert_eq!(
        player.room_id,
        sint_core::types::SystemType::Dormitory.as_u32()
    );
    assert_eq!(player.ap, 2);

    // Try to move to Hallway (7)
    let action = Action::Move {
        to_room: sint_core::types::SystemType::Hallway.as_u32(),
    };

    // This should now FAIL because we are in Lobby
    let res = GameLogic::apply_action(state.clone(), "Player1", action, None);

    match res {
        Ok(_) => {
            panic!("Action should have failed in Lobby!");
        }
        Err(GameError::InvalidAction(msg)) => {
            assert_eq!(msg, "Cannot act during Lobby");
        }
        Err(e) => {
            panic!("Wrong error type: {:?}", e);
        }
    }
}

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
    assert_eq!(
        err.to_string(),
        "Invalid Action: Cannot Pass with 0 AP. Vote ready instead."
    );
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

    let res2 = GameLogic::apply_action(
        state2.clone(),
        "P1",
        Action::VoteReady { ready: true },
        None,
    );
    assert!(res2.is_ok());
    let s2 = res2.unwrap();
    let p1 = s2.players.get("P1").unwrap();
    assert_eq!(p1.ap, 2); // AP Preserved
    assert_eq!(s2.phase, GamePhase::Execution);
}

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
    let has_move_7 = actions.iter().any(|a| {
        matches!(
            a,
            Action::Move { to_room } if *to_room == sint_core::types::SystemType::Hallway.as_u32()
        )
    });
    assert!(has_move_7, "Should be able to move to Room 7");

    // 2. Queue Move to 7
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Move {
            to_room: sint_core::types::SystemType::Hallway.as_u32(),
        },
        None,
    )
    .unwrap();

    // 3. Projected State: Should be in Room 7. Neighbors: [2,3,4,5,6,8,9,10,11]
    let actions = get_valid_actions(&state, "P1");

    // Should NOT be able to move to 7 (already there in projection)
    // Actually Move logic checks neighbors. 7 is not neighbor of 7.
    let has_move_7 = actions.iter().any(|a| {
        matches!(
            a,
            Action::Move { to_room } if *to_room == sint_core::types::SystemType::Hallway.as_u32()
        )
    });
    assert!(
        !has_move_7,
        "Should NOT be able to move to Room 7 (already there)"
    );

    // Should be able to move to 6 (Kitchen)
    let has_move_6 = actions.iter().any(|a| {
        matches!(
            a,
            Action::Move { to_room } if *to_room == sint_core::types::SystemType::Kitchen.as_u32()
        )
    });
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
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Move {
            to_room: sint_core::types::SystemType::Kitchen.as_u32(),
        },
        None,
    )
    .unwrap();

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
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Move {
            to_room: sint_core::types::SystemType::Hallway.as_u32(),
        },
        None,
    )
    .unwrap();

    // Remaining AP: 1.
    let actions = get_valid_actions(&state, "P1");
    let has_move = actions.iter().any(|a| matches!(a, Action::Move { .. }));
    assert!(has_move, "Should still be able to move with 1 AP");

    // Queue another Move (1 AP).
    // From 7 (Hallway) to 6 (Kitchen).
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Move {
            to_room: sint_core::types::SystemType::Kitchen.as_u32(),
        },
        None,
    )
    .unwrap();

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
    if let Some(r) = state
        .map
        .rooms
        .get_mut(&sint_core::types::SystemType::Dormitory.as_u32())
    {
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

#[test]
fn test_undo_middle_of_chain() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // P1 starts in Room 3 (Dormitory). Neighbors include 7 (Hallway).
    // P1 has 2 AP.

    // 1. Queue Move 3 -> 7 (Cost 1)
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
    let id_move_7 = state.proposal_queue[0].id.clone();

    // 2. Queue Move 7 -> 9 (Cost 1) (Room 9 is Bridge, neighbor of 7)
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Move {
            to_room: sint_core::types::SystemType::Bridge.as_u32(),
        },
        None,
    )
    .unwrap();
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
    assert_eq!(
        state.players["P1"].room_id,
        sint_core::types::SystemType::Dormitory.as_u32()
    );

    // 5. Acknowledge Execution (Transition Execution -> TacticalPlanning)
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

    // Phase should loop back to Planning because AP > 0
    assert_eq!(state.phase, GamePhase::TacticalPlanning);
}
