use sint_core::logic::GameLogic;
use sint_core::types::{GameAction, GamePhase};
use sint_solver::driver::GameDriver;

#[test]
fn test_driver_initialization_and_fast_forward() {
    let player_ids = vec!["P1".to_string(), "P2".to_string()];
    let state = GameLogic::new_game(player_ids, 42);

    // Initial state from new_game is Lobby
    assert_eq!(state.phase, GamePhase::Lobby);

    let driver = GameDriver::new(state);

    // stabilize() should have fast-forwarded through Lobby, MorningReport, and EnemyTelegraph
    // to reach TacticalPlanning
    assert_eq!(driver.state.phase, GamePhase::TacticalPlanning);
    assert_eq!(driver.state.turn_count, 1);
}

#[test]
fn test_driver_apply_action() {
    let player_ids = vec!["P1".to_string(), "P2".to_string()];
    let state = GameLogic::new_game(player_ids, 42);
    let mut driver = GameDriver::new(state);

    assert_eq!(driver.state.phase, GamePhase::TacticalPlanning);

    // P1 moves
    let initial_ap = driver.state.players.get("P1").unwrap().ap;
    driver
        .apply("P1", GameAction::Move { to_room: 1 })
        .expect("Apply failed");

    // AP should be reduced
    assert!(driver.state.players.get("P1").unwrap().ap < initial_ap);
}

#[test]
fn test_driver_stabilize_to_next_round() {
    let player_ids = vec!["P1".to_string()];
    let state = GameLogic::new_game(player_ids, 42);
    let mut driver = GameDriver::new(state);

    assert_eq!(driver.state.phase, GamePhase::TacticalPlanning);
    assert_eq!(driver.state.turn_count, 1);

    // P1 passes to end their turn
    driver.apply("P1", GameAction::Pass).expect("Pass failed");

    // stabilize() should have fast-forwarded through Execution, EnemyAction,
    // and the start of next round to reach TacticalPlanning of Round 2
    assert_eq!(driver.state.phase, GamePhase::TacticalPlanning);
    assert_eq!(driver.state.turn_count, 2);
}

#[test]
fn test_driver_partial_ap_readiness() {
    let player_ids = vec!["P1".to_string(), "P2".to_string()];
    let mut state = GameLogic::new_game(player_ids, 42);

    // Manually reach TacticalPlanning
    state.phase = GamePhase::TacticalPlanning;

    // Give P1 some AP and P2 zero AP
    state.players.get_mut("P1").unwrap().ap = 2;
    state.players.get_mut("P2").unwrap().ap = 0;
    state.players.get_mut("P1").unwrap().is_ready = false;
    state.players.get_mut("P2").unwrap().is_ready = false;

    let driver = GameDriver::new(state);

    // P2 should be marked ready because they have 0 AP
    assert!(driver.state.players.get("P2").unwrap().is_ready);
    // P1 should NOT be marked ready because they have AP > 0
    assert!(!driver.state.players.get("P1").unwrap().is_ready);
    // Should still be in TacticalPlanning because P1 is not ready
    assert_eq!(driver.state.phase, GamePhase::TacticalPlanning);
}
