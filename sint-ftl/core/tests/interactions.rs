use sint_core::{GameLogic, Action, GamePhase, CardId};

#[test]
fn test_slippery_and_listing_stacking() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;
    
    // Inject SlipperyDeck (Move=0, Actions=+1)
    let c1 = state.deck.iter().find(|c| c.id == CardId::SlipperyDeck).cloned().unwrap();
    // Inject Listing (Move=0, Actions=*2)
    let c2 = state.deck.iter().find(|c| c.id == CardId::Listing).cloned().unwrap();
    
    // Case A: Slippery THEN Listing
    state.active_situations = vec![c1.clone(), c2.clone()];
    
    // Move: 1 -> Slip(0) -> List(0). Cost 0.
    let cost_move = sint_core::logic::actions::action_cost(&state, "P1", &Action::Move { to_room: 0 });
    assert_eq!(cost_move, 0, "Move should be free (0 -> 0)");
    
    // Bake: 1 -> Slip(2) -> List(4). Cost 4.
    let cost_bake = sint_core::logic::actions::action_cost(&state, "P1", &Action::Bake);
    assert_eq!(cost_bake, 4, "Bake: 1 -> +1(2) -> *2(4)");
    
    // Case B: Listing THEN Slippery
    state.active_situations = vec![c2.clone(), c1.clone()];
    
    // Move: 1 -> List(0) -> Slip(0). Cost 0.
    let cost_move = sint_core::logic::actions::action_cost(&state, "P1", &Action::Move { to_room: 0 });
    assert_eq!(cost_move, 0, "Move should be free");
    
    // Bake: 1 -> List(2) -> Slip(3). Cost 3.
    let cost_bake = sint_core::logic::actions::action_cost(&state, "P1", &Action::Bake);
    assert_eq!(cost_bake, 3, "Bake: 1 -> *2(2) -> +1(3)");
}

#[test]
fn test_serialization_roundtrip() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    // Mutate state
    state.phase = GamePhase::Execution;
    // Add a card
    state.active_situations.push(state.deck[0].clone());
    // Damage player
    if let Some(p) = state.players.get_mut("P1") {
        p.hp = 1;
    }
    
    // Serialize
    let json = serde_json::to_string(&state).unwrap();
    
    // Deserialize
    let state2: sint_core::GameState = serde_json::from_str(&json).unwrap();
    
    // Compare
    assert_eq!(state, state2, "State should match after roundtrip");
}
