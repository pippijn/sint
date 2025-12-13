use sint_core::{Action, GameAction, GameLogic, GamePhase};

#[test]
fn test_client_logic() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Verify we can perform logic
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 2;
    }

    // Attempt pass
    let res = GameLogic::apply_action(state, "P1", Action::Game(GameAction::Pass), None);
    assert!(res.is_ok(), "Client should be able to execute core logic");
}
