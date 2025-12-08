use sint_core::{Action, GameLogic, GamePhase, HazardType};

#[test]
fn test_fire_damage() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    
    // Setup: P1 in Kitchen (6), Fire in Kitchen
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 6;
        p.ap = 0; // Force end of turn
        p.is_ready = true;
    }
    
    if let Some(room) = state.map.rooms.get_mut(&6) {
        room.hazards.push(HazardType::Fire);
    }
    
    let initial_hull = state.hull_integrity;
    let initial_hp = state.players["P1"].hp;

    // Transition: Execution -> EnemyAction (Triggers resolve_hazards)
    state.phase = GamePhase::Execution; 
    
    // We need to trigger "VoteReady" or "Pass" to advance phase, but since AP is 0,
    // apply_action(VoteReady) should trigger advance_phase to EnemyAction.
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    
    assert_eq!(state.phase, GamePhase::EnemyAction);
    
    // Check Damage
    assert_eq!(state.hull_integrity, initial_hull - 1, "Hull should take fire damage");
    assert_eq!(state.players["P1"].hp, initial_hp - 1, "Player should take fire damage");
}

#[test]
fn test_extinguish_fire() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;
    
    // Setup: P1 in Kitchen (6), Fire in Kitchen
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 6;
        p.ap = 2;
    }
    if let Some(room) = state.map.rooms.get_mut(&6) {
        room.hazards.push(HazardType::Fire);
    }
    
    // Action: Extinguish
    state = GameLogic::apply_action(state, "P1", Action::Extinguish, None).unwrap();
    
    // Check AP cost
    assert_eq!(state.players["P1"].ap, 1);
    
    // Advance to Execution to resolve
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::Execution);
    
    // Trigger Resolution (VoteReady again or implicit if auto-trigger? apply_action doesn't auto-resolve logic unless phase advanced)
    // Wait, advance_phase(TacticalPlanning) -> calls resolve_proposal_queue immediately!
    // So Extinguish should be done.
    
    if let Some(room) = state.map.rooms.get(&6) {
        assert!(room.hazards.is_empty(), "Fire should be extinguished");
    }
}

#[test]
fn test_repair_water() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;
    
    // Setup: Water in Kitchen
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 6;
    }
    if let Some(room) = state.map.rooms.get_mut(&6) {
        room.hazards.push(HazardType::Water);
    }
    
    // Action: Repair
    state = GameLogic::apply_action(state, "P1", Action::Repair, None).unwrap();
    
    // Advance to resolve
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    
    if let Some(room) = state.map.rooms.get(&6) {
        assert!(room.hazards.is_empty(), "Water should be repaired");
    }
}
