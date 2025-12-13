use sint_core::{
    GameLogic,
    logic::resolution::process_round_end,
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

    state.active_situations.push(card);

    // Verify Free Moves to Hallway (Room 0)
    let cost =
        sint_core::logic::actions::action_cost(&state, "P1", &GameAction::Move { to_room: 0 });
    assert_eq!(cost, 0);

    // Other actions +1
    let kitchen = sint_core::logic::find_room_with_system_in_map(
        &state.map,
        sint_core::types::SystemType::Kitchen,
    )
    .unwrap();
    state.players.get_mut("P1").unwrap().room_id = kitchen;

    let cost_bake = sint_core::logic::actions::action_cost(&state, "P1", &GameAction::Bake);
    assert_eq!(cost_bake, 2);

    // Check Lifecycle
    // Round 1
    process_round_end(&mut state);
    // Round 2
    process_round_end(&mut state);
    // Round 3
    process_round_end(&mut state);

    // Should be gone
    let still_active = state
        .active_situations
        .iter()
        .any(|c| c.id == CardId::SlipperyDeck);
    assert!(!still_active, "Slippery Deck should expire after 3 rounds");
}
