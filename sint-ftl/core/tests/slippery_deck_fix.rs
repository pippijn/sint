use sint_core::{
    GameLogic,
    types::{CardId, GameAction, GamePhase},
};

fn new_test_game(players: Vec<String>) -> sint_core::types::GameState {
    let mut state = GameLogic::new_game(players, 12345);
    state.deck.clear();
    state
}

#[test]
fn test_slippery_deck_lifecycle() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    state.phase = GamePhase::MorningReport;

    use sint_core::logic::cards::get_behavior;
    let behavior = get_behavior(CardId::SlipperyDeck);
    let card = behavior.get_struct();

    // Check type - Should be Timebomb after fix
    // For now, in TDD, we assert what we WANT.
    // But since I'm modifying existing code, I can write the test to expect the NEW behavior
    // and fail if it's the old one.

    state.active_situations.push(card);

    // Verify 0 AP cost for move
    let cost_move =
        sint_core::logic::actions::action_cost(&state, "P1", &GameAction::Move { to_room: 0 });
    assert_eq!(cost_move, 0);

    // Verify +1 AP cost for other actions
    let kitchen = sint_core::logic::find_room_with_system_in_map(
        &state.map,
        sint_core::types::SystemType::Kitchen,
    )
    .unwrap();
    state.players.get_mut("P1").unwrap().room_id = kitchen;
    let cost_bake = sint_core::logic::actions::action_cost(&state, "P1", &GameAction::Bake);
    assert_eq!(cost_bake, 2); // Base 1 + 1

    // Advance 3 rounds to check expiry (Simulated)
    // We need to call on_round_end manually as the engine does

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
        .any(|c| c.id == CardId::SlipperyDeck);
    assert!(!still_active, "Slippery Deck should expire after 3 rounds");
}
