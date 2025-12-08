use sint_core::{Action, GameLogic, GamePhase, HazardType, PlayerStatus};

#[test]
fn test_game_over_hull_destruction() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::Execution;

    // Set Hull to 1
    state.hull_integrity = 1;

    // Set Hazard in Kitchen
    if let Some(r) = state.map.rooms.get_mut(&6) {
        r.hazards.push(HazardType::Fire);
    }

    // Prepare player to finish execution
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 0;
        p.is_ready = true;
    }

    // Trigger Advance (Execution -> EnemyAction -> Game Over Check)
    // Note: apply_action doesn't run advance_phase unless all players are ready.
    // VoteReady logic calls advance_phase if consensus.
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

    // Logic:
    // 1. resolve_hazards runs. Fire deals 1 damage to Hull. Hull = 0.
    // 2. advance_phase checks hull <= 0.
    // 3. state.phase = GameOver.

    assert_eq!(state.phase, GamePhase::GameOver);
    assert_eq!(state.hull_integrity, 0);
}

#[test]
fn test_game_over_crew_wipe() {
    let mut state = GameLogic::new_game(vec!["P1".to_string(), "P2".to_string()], 12345);
    state.phase = GamePhase::Execution;

    // Set P1 and P2 HP to 1, put them in Fire
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 6;
        p.hp = 1;
        p.ap = 0;
        p.is_ready = true;
    }
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = 6;
        p.hp = 1;
        p.ap = 0;
        p.is_ready = true;
    }

    if let Some(r) = state.map.rooms.get_mut(&6) {
        r.hazards.push(HazardType::Fire);
    }

    // Advance
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    state = GameLogic::apply_action(state, "P2", Action::VoteReady { ready: true }, None).unwrap();

    // Logic:
    // 1. resolve_hazards runs. Both players take 1 damage -> 0 HP -> Fainted.
    // 2. Check crew_wiped.
    // 3. state.phase = GameOver.

    assert!(state.players["P1"].status.contains(&PlayerStatus::Fainted));
    assert!(state.players["P2"].status.contains(&PlayerStatus::Fainted));
    assert_eq!(state.phase, GamePhase::GameOver);
}
