use sint_core::{Action, CardType, GameLogic, GamePhase, CardId};

#[test]
fn test_card_c04_slippery_deck() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Inject C04
    let card = state.deck.iter().find(|c| c.id == CardId::SlipperyDeck).cloned().unwrap();
    state.active_situations.push(card);

    // Setup P1
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 6; // Kitchen
        p.ap = 2;
    }

    // 1. Move should be FREE (0 AP)
    state = GameLogic::apply_action(state, "P1", Action::Move { to_room: 7 }, None).unwrap();
    assert_eq!(state.players["P1"].ap, 2, "Move should cost 0 AP with Slippery Deck");

    // 2. Bake should cost +1 (Total 2 AP)
    // Bake base cost 1 -> +1 = 2.
    // P1 currently in 6 (Kitchen), Moving to 7 (Hallway).
    // Can't bake in Hallway.
    // Let's Move BACK to 6 (Free) then Bake.
    state = GameLogic::apply_action(state, "P1", Action::Move { to_room: 6 }, None).unwrap();
    assert_eq!(state.players["P1"].ap, 2);

    state = GameLogic::apply_action(state, "P1", Action::Bake, None).unwrap();
    assert_eq!(state.players["P1"].ap, 0, "Bake should cost 2 AP (1+1)");
}

#[test]
fn test_card_c11_mutiny_explosion() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    
    // Inject C11 Mutiny with 1 round left
    let mut card = state.deck.iter().find(|c| c.id == CardId::Mutiny).cloned().unwrap();
    card.card_type = CardType::Timebomb { rounds_left: 1 };
    state.active_situations.push(card);
    
    let initial_hull = state.hull_integrity;
    
    // Trigger Round End (Transition Execution -> EnemyAction)
    state.phase = GamePhase::Execution;
    if let Some(p) = state.players.get_mut("P1") { p.ap = 0; p.is_ready = true; }
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    
    assert_eq!(state.phase, GamePhase::EnemyAction);
    
    // Mutiny triggers at End of Round (EnemyAction -> MorningReport)
    // Wait, let's check `logic/cards/impls/c11_mutiny.rs`: `on_round_end`.
    // `advance_phase` calls `on_round_end` when exiting `EnemyAction`.
    
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::MorningReport);
    
    // Should have exploded (-10 Hull)
    assert_eq!(state.hull_integrity, initial_hull - 10, "Mutiny should deal 10 hull damage");
    assert!(state.active_situations.iter().all(|c| c.id != CardId::Mutiny), "Mutiny card should be removed");
}

#[test]
fn test_card_c21_sing_a_song() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    
    // Setup Hazards
    use sint_core::HazardType;
    state.map.rooms.get_mut(&6).unwrap().hazards.push(HazardType::Fire);
    state.map.rooms.get_mut(&4).unwrap().hazards.push(HazardType::Water);
    
    // Get C21
    let card = state.deck.iter().find(|c| c.id == CardId::SingASong).cloned().unwrap();
    
    // Simulate Draw (Manual activation)
    use sint_core::logic::cards::get_behavior;
    get_behavior(card.id).on_activate(&mut state);
    
    // Verify hazards removed
    assert!(state.map.rooms[&6].hazards.is_empty(), "Fire should be removed");
    assert!(state.map.rooms[&4].hazards.is_empty(), "Water should be removed");
}
