use sint_core::{
    types::{CardId, GamePhase},
    GameLogic,
};

fn new_test_game(players: Vec<String>) -> sint_core::types::GameState {
    let mut state = GameLogic::new_game(players, 12345);
    state.deck.clear();
    state
}

#[test]
fn test_sugar_rush_lifecycle() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    state.phase = GamePhase::MorningReport;

    use sint_core::logic::cards::get_behavior;
    let behavior = get_behavior(CardId::SugarRush);
    let card = behavior.get_struct();

    state.active_situations.push(card);

    // Verify Free Moves (up to 5)
    // We can't easily check 'modify_action_cost' statefully without queuing actions in a real game loop context,
    // but we can check if it expires.

    // Check Lifecycle
    // Round 1
    behavior.on_round_end(&mut state);
    // Round 2
    behavior.on_round_end(&mut state);
    // Round 3
    behavior.on_round_end(&mut state);

    // Should be gone
    let still_active = state
        .active_situations
        .iter()
        .any(|c| c.id == CardId::SugarRush);
    assert!(!still_active, "Sugar Rush should expire after 3 rounds");
}
