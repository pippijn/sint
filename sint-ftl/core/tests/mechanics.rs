use sint_core::{Action, GameLogic, GamePhase, HazardType, ItemType, PlayerStatus};

#[test]
fn test_boss_progression() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::Execution;

    // Check Initial Boss (Level 0)
    assert_eq!(state.boss_level, 0);
    assert_eq!(state.enemy.name, "The Petty Thief");

    // Give P1 ammo and put in Cannons
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sint_core::types::SystemType::Cannons.as_u32();
        p.inventory = vec![ItemType::Peppernut; 100]; // Infinite ammo
        p.ap = 0;
        p.is_ready = true;
    }

    // Force Boss HP to 1
    state.enemy.hp = 1;

    let mut attempts = 0;
    while state.boss_level == 0 && attempts < 100 {
        // Queue Shot
        state.proposal_queue.push(sint_core::ProposedAction {
            id: "kill_shot".to_string(),
            player_id: "P1".to_string(),
            action: Action::Shoot,
        });

        // Ensure HP is 1
        state.enemy.hp = 1;

        sint_core::logic::resolution::resolve_proposal_queue(&mut state, false);
        attempts += 1;
    }

    assert!(state.boss_level == 1, "Should progress to Level 1");
    assert_eq!(state.enemy.name, "The Monster");
    assert_eq!(state.enemy.hp, 10);
}

#[test]
fn test_victory_condition() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::Execution;

    // Jump to Level 3 (The Kraken)
    state.boss_level = 3;
    state.enemy = sint_core::logic::get_boss(3);
    assert_eq!(state.enemy.name, "The Kraken");

    // Set Kraken HP to 1
    state.enemy.hp = 1;

    // P1 Shoot setup
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sint_core::types::SystemType::Cannons.as_u32();
        p.inventory = vec![ItemType::Peppernut; 100];
    }

    // Kill Kraken
    let mut attempts = 0;
    while state.phase != GamePhase::Victory && attempts < 100 {
        state.proposal_queue.push(sint_core::ProposedAction {
            id: "kill_shot".to_string(),
            player_id: "P1".to_string(),
            action: Action::Shoot,
        });
        state.enemy.hp = 1;
        sint_core::logic::resolution::resolve_proposal_queue(&mut state, false);
        attempts += 1;
    }

    assert_eq!(state.phase, GamePhase::Victory);
}

#[test]
fn test_shoot_mechanics() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Setup: P1 in Cannons (8), Has Ammo
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sint_core::types::SystemType::Cannons.as_u32();
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
    assert!(
        state.players["P1"].inventory.is_empty(),
        "Ammo should be consumed"
    );

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
        p.room_id = sint_core::types::SystemType::Engine.as_u32();
        p.ap = 2;
    }

    assert_eq!(state.shields_active, false);

    // Action: RaiseShields
    state = GameLogic::apply_action(state, "P1", Action::RaiseShields, None).unwrap();

    // AP Cost is 2
    assert_eq!(state.players["P1"].ap, 0);

    // Advance to resolve
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

    assert_eq!(
        state.shields_active, true,
        "Shields should be active after resolution"
    );
}

#[test]
fn test_evasion_activation() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Setup: P1 in Bridge (9)
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sint_core::types::SystemType::Bridge.as_u32();
        p.ap = 2;
    }

    assert_eq!(state.evasion_active, false);

    // Action: EvasiveManeuvers
    state = GameLogic::apply_action(state, "P1", Action::EvasiveManeuvers, None).unwrap();

    // AP Cost is 2
    assert_eq!(state.players["P1"].ap, 0);

    // Advance to resolve
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

    assert_eq!(
        state.evasion_active, true,
        "Evasion should be active after resolution"
    );
}

#[test]
fn test_shields_block_damage() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);

    // Force Shields Active
    state.shields_active = true;

    // Setup Enemy Attack
    use sint_core::{AttackEffect, EnemyAttack};
    state.enemy.next_attack = Some(EnemyAttack {
        target_room: sint_core::types::SystemType::Kitchen.as_u32(),
        effect: AttackEffect::Fireball,
    });

    let initial_hull = state.hull_integrity;

    // Trigger resolution via EnemyAction phase
    // We can call logic directly or transition phase.
    // Transitioning is safer integration test.
    state.phase = GamePhase::Execution;
    // Set AP to 0 so we don't go back to Planning
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 0;
        p.is_ready = true;
    }

    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

    assert_eq!(state.phase, GamePhase::EnemyAction);

    // Check Damage Blocked
    assert_eq!(
        state.hull_integrity, initial_hull,
        "Shields should block damage"
    );
    // Check Hazard
    if let Some(room) = state
        .map
        .rooms
        .get(&sint_core::types::SystemType::Kitchen.as_u32())
    {
        assert!(room.hazards.is_empty(), "Shields should prevent fire spawn");
    }
}

#[test]
fn test_game_over_hull_destruction() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::Execution;

    // Set Hull to 1
    state.hull_integrity = 1;

    // Set Hazard in Kitchen
    if let Some(r) = state
        .map
        .rooms
        .get_mut(&sint_core::types::SystemType::Kitchen.as_u32())
    {
        r.hazards.push(HazardType::Fire);
    }

    // Prepare player to finish execution
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 0;
        p.is_ready = true;
    }

    // Trigger Advance (Execution -> EnemyAction -> Game Over Check)
    // Note: apply_action doesn't run advance_phase unless all players are ready.
    // VoteReady logic calls advance_phase if consensus.
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

    // Logic:
    // 1. resolve_hazards runs. Fire deals 1 damage to Hull. Hull = 0.
    // 2. advance_phase checks hull <= 0.
    // 3. state.phase = GameOver.

    assert_eq!(state.phase, GamePhase::GameOver);
    assert_eq!(state.hull_integrity, 0);
}

#[test]
fn test_game_over_crew_wipe() {
    let mut state = GameLogic::new_game(vec!["P1".to_string(), "P2".to_string()], 12345);
    state.phase = GamePhase::Execution;

    // Set P1 and P2 HP to 1, put them in Fire
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sint_core::types::SystemType::Kitchen.as_u32();
        p.hp = 1;
        p.ap = 0;
        p.is_ready = true;
    }
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = sint_core::types::SystemType::Kitchen.as_u32();
        p.hp = 1;
        p.ap = 0;
        p.is_ready = true;
    }

    if let Some(r) = state
        .map
        .rooms
        .get_mut(&sint_core::types::SystemType::Kitchen.as_u32())
    {
        r.hazards.push(HazardType::Fire);
    }

    // Advance
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    state = GameLogic::apply_action(state, "P2", Action::VoteReady { ready: true }, None).unwrap();

    // Logic:
    // 1. resolve_hazards runs. Both players take 1 damage -> 0 HP -> Fainted.
    // 2. Check crew_wiped.
    // 3. state.phase = GameOver.

    assert!(state.players["P1"].status.contains(&PlayerStatus::Fainted));
    assert!(state.players["P2"].status.contains(&PlayerStatus::Fainted));
    assert_eq!(state.phase, GamePhase::GameOver);
}

#[test]
fn test_simulation_masks_rng_outcome() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Setup: P1 in Cannons (8), Has Ammo.
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 8;
        p.inventory.push(ItemType::Peppernut);
        p.ap = 2;
    }

    let initial_hp = state.enemy.hp;

    // 1. Propose Action (Shoot)
    state.proposal_queue.push(sint_core::ProposedAction {
        id: "test".to_string(),
        player_id: "P1".to_string(),
        action: Action::Shoot,
    });

    // 2. "Simulate" by projecting the state
    // This is what get_valid_actions does internally to validate subsequent moves.
    let mut projected_state = state.clone();
    sint_core::logic::resolution::resolve_proposal_queue(&mut projected_state, true); // true = simulation

    // 3. Check Projected Outcome
    println!(
        "Initial HP: {}, Projected HP: {}",
        initial_hp, projected_state.enemy.hp
    );

    // Assert that we CANNOT see the result.
    // HP should be UNCHANGED in projection (RNG Masked).
    assert_eq!(
        projected_state.enemy.hp, initial_hp,
        "HP should NOT change in simulation"
    );

    // But Ammo should still be consumed (Deterministic cost).
    assert!(
        projected_state.players["P1"].inventory.is_empty(),
        "Ammo consumed in projection"
    );
}

#[test]
fn test_single_player_start() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    assert_eq!(state.phase, GamePhase::Lobby);

    // P1 votes ready
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

    // Should start immediately
    assert_eq!(state.phase, GamePhase::MorningReport);
}

#[test]
fn test_multi_player_consensus() {
    let mut state = GameLogic::new_game(vec!["P1".to_string(), "P2".to_string()], 12345);

    // P1 votes ready
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::Lobby); // Not yet

    // P2 votes ready
    state = GameLogic::apply_action(state, "P2", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::MorningReport); // Now
}

#[test]
fn test_multi_player_join_late() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);

    // P1 votes ready... wait, if P1 votes ready, it starts immediately if only 1 player.
    // So we need P1 to NOT be ready, then P2 joins.

    // 1. P1 joins (is_ready=false)
    assert_eq!(state.phase, GamePhase::Lobby);

    // 2. P2 joins
    let join_action = Action::Join {
        name: "P2".to_string(),
    };
    state = GameLogic::apply_action(state, "P2", join_action, None).unwrap();

    assert!(state.players.contains_key("P2"));
    assert_eq!(state.phase, GamePhase::Lobby);

    // 3. P1 votes ready
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::Lobby); // P2 not ready

    // 4. P2 votes ready
    state = GameLogic::apply_action(state, "P2", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::MorningReport);
}
