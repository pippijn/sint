use sint_core::{Action, GameLogic, GamePhase, ItemType};

#[test]
fn test_shoot_mechanics() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;
    
    // Setup: P1 in Cannons (8), Has Ammo
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 8;
        p.inventory.push(ItemType::Peppernut);
        p.ap = 2;
    }
    
    // Action: Shoot
    state = GameLogic::apply_action(state, "P1", Action::Shoot, None).unwrap();
    
    // AP deducted
    assert_eq!(state.players["P1"].ap, 1);
    
    // Advance to resolve
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    
    // Verify Ammo consumed
    assert!(state.players["P1"].inventory.is_empty(), "Ammo should be consumed");
    
    // Note: Hit/Miss is RNG. We can't strictly assert HP change without mocking RNG 
    // or forcing a seed that hits. 
    // But we know standard threshold is 3/6.
    // 12345 seed might hit or miss.
    // If we wanted to be sure, we'd check logic::resolution::resolve_proposal_queue.
}

#[test]
fn test_shields_activation() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;
    
    // Setup: P1 in Engine (5)
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 5;
        p.ap = 2;
    }
    
    assert_eq!(state.shields_active, false);
    
    // Action: RaiseShields
    state = GameLogic::apply_action(state, "P1", Action::RaiseShields, None).unwrap();
    
    // AP Cost is 2
    assert_eq!(state.players["P1"].ap, 0);
    
    // Advance to resolve
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    
    assert_eq!(state.shields_active, true, "Shields should be active after resolution");
}

#[test]
fn test_evasion_activation() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;
    
    // Setup: P1 in Bridge (9)
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 9;
        p.ap = 2;
    }
    
    assert_eq!(state.evasion_active, false);
    
    // Action: EvasiveManeuvers
    state = GameLogic::apply_action(state, "P1", Action::EvasiveManeuvers, None).unwrap();
    
    // AP Cost is 2
    assert_eq!(state.players["P1"].ap, 0);
    
    // Advance to resolve
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    
    assert_eq!(state.evasion_active, true, "Evasion should be active after resolution");
}

#[test]
fn test_shields_block_damage() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    
    // Force Shields Active
    state.shields_active = true;
    
    // Setup Enemy Attack
    use sint_core::{EnemyAttack, AttackEffect};
    state.enemy.next_attack = Some(EnemyAttack {
        target_room: 6,
        effect: AttackEffect::Fireball,
    });
    
    let initial_hull = state.hull_integrity;
    
    // Trigger resolution via EnemyAction phase
    // We can call logic directly or transition phase.
    // Transitioning is safer integration test.
    state.phase = GamePhase::Execution;
    // Set AP to 0 so we don't go back to Planning
    if let Some(p) = state.players.get_mut("P1") { p.ap = 0; p.is_ready = true; }
    
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    
    assert_eq!(state.phase, GamePhase::EnemyAction);
    
    // Check Damage Blocked
    assert_eq!(state.hull_integrity, initial_hull, "Shields should block damage");
    // Check Hazard
    if let Some(room) = state.map.rooms.get(&6) {
        assert!(room.hazards.is_empty(), "Shields should prevent fire spawn");
    }
}
