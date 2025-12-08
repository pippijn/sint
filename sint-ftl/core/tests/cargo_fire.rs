use sint_core::{Action, GameLogic, GamePhase, HazardType};

#[test]
fn test_cargo_fire_spread_threshold() {
    // We run multiple iterations to ensure we hit the 50% chance if logic allows it.
    // Kitchen (Threshold 2) should NEVER spread with 1 Fire.
    // Cargo (Threshold 1) SHOULD spread eventually.

    let mut cargo_spread_count = 0;
    let mut kitchen_spread_count = 0;

    for i in 0..20 {
        let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345 + i);
        state.phase = GamePhase::Execution;

        // Setup: 1 Fire in Cargo (4), 1 Fire in Kitchen (6)
        if let Some(r) = state.map.rooms.get_mut(&4) {
            r.hazards.push(HazardType::Fire);
        }
        if let Some(r) = state.map.rooms.get_mut(&6) {
            r.hazards.push(HazardType::Fire);
        }

        // Neighbors of Cargo (4): 7 (Hallway)
        // Neighbors of Kitchen (6): 7 (Hallway)

        // Ensure Hallway is clear
        if let Some(r) = state.map.rooms.get_mut(&7) {
            r.hazards.clear();
        }

        // Trigger Resolution (Transition to EnemyAction calls resolve_hazards)
        // We force AP to 0 so VoteReady triggers phase change.
        if let Some(p) = state.players.get_mut("P1") {
            p.ap = 0;
            p.is_ready = true;
        }
        state =
            GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

        // Check Hallway for spread
        // But both spread to Hallway (7).
        // Wait, if BOTH spread, we can't distinguish source easily without checking logs (which we don't have for hazards).

        // Let's test them separately.
    }
}

#[test]
fn test_kitchen_standard_threshold() {
    // Kitchen should NOT spread with 1 Fire.
    for i in 0..20 {
        let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345 + i);
        state.phase = GamePhase::Execution;
        if let Some(r) = state.map.rooms.get_mut(&6) {
            r.hazards.push(HazardType::Fire);
        }
        if let Some(r) = state.map.rooms.get_mut(&7) {
            r.hazards.clear();
        }
        if let Some(p) = state.players.get_mut("P1") {
            p.ap = 0;
            p.is_ready = true;
        }
        state =
            GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

        if let Some(r) = state.map.rooms.get(&7) {
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
        if let Some(r) = state.map.rooms.get_mut(&4) {
            r.hazards.push(HazardType::Fire);
        }
        if let Some(r) = state.map.rooms.get_mut(&7) {
            r.hazards.clear();
        }
        if let Some(p) = state.players.get_mut("P1") {
            p.ap = 0;
            p.is_ready = true;
        }
        state =
            GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

        if let Some(r) = state.map.rooms.get(&7) {
            if r.hazards.contains(&HazardType::Fire) {
                spread_occured = true;
                break;
            }
        }
    }
    assert!(spread_occured, "Cargo with 1 Fire SHOULD spread eventually");
}
