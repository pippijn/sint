use sint_core::{Action, GameLogic, GamePhase, HazardType};

#[test]
fn test_kitchen_standard_threshold() {
    // Kitchen should NOT spread with 1 Fire.
    for i in 0..20 {
        let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345 + i);
        state.phase = GamePhase::Execution;
        if let Some(r) = state.map.rooms.get_mut(&sint_core::logic::ROOM_KITCHEN) {
            r.hazards.push(HazardType::Fire);
        }
        if let Some(r) = state.map.rooms.get_mut(&sint_core::logic::ROOM_HALLWAY) {
            r.hazards.clear();
        }
        if let Some(p) = state.players.get_mut("P1") {
            p.ap = 0;
            p.is_ready = true;
        }
        state =
            GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

        if let Some(r) = state.map.rooms.get(&sint_core::logic::ROOM_HALLWAY) {
            assert!(
                r.hazards.is_empty(),
                "Standard room with 1 Fire should NOT spread"
            );
        }
    }
}

#[test]
fn test_cargo_lower_threshold() {
    // Cargo SHOULD spread with 1 Fire (approx 50% of time).
    let mut spread_occured = false;

    for i in 0..20 {
        let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345 + i);
        state.phase = GamePhase::Execution;
        if let Some(r) = state.map.rooms.get_mut(&sint_core::logic::ROOM_CARGO) {
            r.hazards.push(HazardType::Fire);
        }
        if let Some(r) = state.map.rooms.get_mut(&sint_core::logic::ROOM_HALLWAY) {
            r.hazards.clear();
        }
        if let Some(p) = state.players.get_mut("P1") {
            p.ap = 0;
            p.is_ready = true;
        }
        state =
            GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

        if let Some(r) = state.map.rooms.get(&sint_core::logic::ROOM_HALLWAY) {
            if r.hazards.contains(&HazardType::Fire) {
                spread_occured = true;
                break;
            }
        }
    }
    assert!(spread_occured, "Cargo with 1 Fire SHOULD spread eventually");
}
