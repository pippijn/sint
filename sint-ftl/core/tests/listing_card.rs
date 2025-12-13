use sint_core::{
    GameLogic,
    logic::{actions::action_cost, cards::get_behavior, find_room_with_system_in_map},
    types::*,
};

fn new_test_game(players: Vec<String>) -> GameState {
    let mut state = GameLogic::new_game(players, 12345);
    state.deck.clear(); // Remove RNG events
    state
}

#[test]
fn test_listing_card_lifecycle() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    state.phase = GamePhase::MorningReport;

    let behavior = get_behavior(CardId::Listing);
    let card = behavior.get_struct();

    // Check type
    if let CardType::Timebomb { rounds_left } = card.card_type {
        assert_eq!(rounds_left, 3);
    } else {
        panic!("ListingCard should be a Timebomb");
    }

    state.active_situations.push(card);

    // Apply start of round effect (+5 AP)
    // Note: new_game gives 2 AP by default.
    behavior.on_round_start(&mut state);
    // Base AP is 2. +5 = 7.
    assert_eq!(state.players["P1"].ap, 7);

    // Verify costs
    let cost_move = action_cost(&state, "P1", &GameAction::Move { to_room: 0 });
    assert_eq!(cost_move, 1);

    // Other actions (e.g. Bake) usually 1, should be 2.
    // Need to put player in Kitchen first
    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();
    state.players.get_mut("P1").unwrap().room_id = kitchen;

    let cost_bake = action_cost(&state, "P1", &GameAction::Bake);
    assert_eq!(cost_bake, 2);

    // Advance 3 rounds to check expiry
    // Round 1 End
    behavior.on_round_end(&mut state);
    {
        let c = state
            .active_situations
            .iter()
            .find(|c| c.id == CardId::Listing)
            .unwrap();
        if let CardType::Timebomb { rounds_left } = c.card_type {
            assert_eq!(rounds_left, 2);
        }
    }

    // Round 2 End
    behavior.on_round_end(&mut state);
    {
        let c = state
            .active_situations
            .iter()
            .find(|c| c.id == CardId::Listing)
            .unwrap();
        if let CardType::Timebomb { rounds_left } = c.card_type {
            assert_eq!(rounds_left, 1);
        }
    }

    // Round 3 End
    behavior.on_round_end(&mut state);

    // Should be gone
    assert!(
        state
            .active_situations
            .iter()
            .all(|c| c.id != CardId::Listing)
    );
}
