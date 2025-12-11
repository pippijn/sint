use sint_core::{
    types::{Action, GameAction, GamePhase, HazardType, ItemType, MetaAction, PlayerStatus},
    GameLogic,
};

#[test]
fn test_cannon_hit() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let cannons = sint_core::logic::find_room_with_system_in_map(&state.map, sint_core::types::SystemType::Cannons).unwrap();

    // Setup: P1 in Cannons, Has Nut. Enemy HP 5.
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = cannons;
        p.inventory.push(ItemType::Peppernut);
    }
    // Set seed to guarantee hit (threshold 3). 12345 next rand is > 3?
    // We can't easily control rand unless we mock it or brute force seed.
    // Or we assume 12345 works (it was used in original tests).

    let res = GameLogic::apply_action(state.clone(), "P1", Action::Game(GameAction::Shoot), None);
    assert!(res.is_ok());

    let mut state = res.unwrap();
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    // Check Enemy HP
    // Original test says "12345" works?
    // Let's assume it hits.
    // If it fails, I might need to adjust seed or logic.
    assert!(state.enemy.hp < 5);
}

#[test]
fn test_shields_block_damage() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let engine = sint_core::logic::find_room_with_system_in_map(&state.map, sint_core::types::SystemType::Engine).unwrap();
    let kitchen = sint_core::logic::find_room_with_system_in_map(&state.map, sint_core::types::SystemType::Kitchen).unwrap();

    // P1 in Engine
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = engine;
        p.ap = 2;
    }

    // Action: Raise Shields
    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::RaiseShields),
        None,
    );
    assert!(res.is_ok());

    let mut state = res.unwrap();
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    assert!(state.shields_active);

    // Simulate Enemy Attack
    state.phase = GamePhase::EnemyAction;
    // Set up attack
    state.enemy.next_attack = Some(sint_core::types::EnemyAttack {
        target_room: kitchen,
        target_system: Some(sint_core::types::SystemType::Kitchen),
        effect: sint_core::types::AttackEffect::Fireball,
    });

    sint_core::logic::resolution::resolve_enemy_attack(&mut state);

    // Should block damage -> Hull remains 20 (or whatever it was)
    assert_eq!(state.hull_integrity, 20);
    // No fire
    assert!(state.map.rooms[&kitchen].hazards.is_empty());
}

#[test]
fn test_evasion_blocks_hit() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let bridge = sint_core::logic::find_room_with_system_in_map(&state.map, sint_core::types::SystemType::Bridge).unwrap();
    let kitchen = sint_core::logic::find_room_with_system_in_map(&state.map, sint_core::types::SystemType::Kitchen).unwrap();

    // P1 in Bridge
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = bridge;
        p.ap = 2;
    }

    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::EvasiveManeuvers),
        None,
    );
    assert!(res.is_ok());

    let mut state = res.unwrap();
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    assert!(state.evasion_active);

    // Attack
    state.phase = GamePhase::EnemyAction;
    state.enemy.next_attack = Some(sint_core::types::EnemyAttack {
        target_room: kitchen,
        target_system: Some(sint_core::types::SystemType::Kitchen),
        effect: sint_core::types::AttackEffect::Fireball,
    });

    sint_core::logic::resolution::resolve_enemy_attack(&mut state);

    assert_eq!(state.hull_integrity, 20);
    assert!(state.map.rooms[&kitchen].hazards.is_empty());
}

#[test]
fn test_boss_progression() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let cannons = sint_core::logic::find_room_with_system_in_map(&state.map, sint_core::types::SystemType::Cannons).unwrap();

    // Set Boss HP to 1
    state.enemy.hp = 1;
    state.boss_level = 0; // Petty Thief

    // P1 Shoot
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = cannons;
        p.inventory.push(ItemType::Peppernut);
    }

    let res = GameLogic::apply_action(state.clone(), "P1", Action::Game(GameAction::Shoot), None);
    let mut state = res.unwrap();
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    // Check Boss Level increased
    assert_eq!(state.boss_level, 1);
    assert_eq!(state.enemy.name, "The Monster"); // Level 1 Boss
}

#[test]
fn test_game_over_hull() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.hull_integrity = 1;
    state.phase = GamePhase::EnemyAction;

    let kitchen = sint_core::logic::find_room_with_system_in_map(&state.map, sint_core::types::SystemType::Kitchen).unwrap();

    // Trigger hazard damage
    if let Some(r) = state.map.rooms.get_mut(&kitchen) {
        r.hazards.push(HazardType::Fire);
    }

    sint_core::logic::resolution::resolve_hazards(&mut state);
    // Logic: Hull 1 -> 0.
    // resolve_hazards DOES NOT check Game Over. `advance_phase` does.
    // Wait, `advance_phase` handles Execution -> EnemyAction.
    // But inside `Execution`, it checks Game Over?
    // `resolve_hazards` runs in `Execution` (end of).
    // Yes:
    /*
        state.phase = GamePhase::EnemyAction;
        resolution::resolve_enemy_attack(&mut state);
        resolution::resolve_hazards(&mut state);
        let hull_destroyed = state.hull_integrity <= 0;
        if hull_destroyed ... { state.phase = GamePhase::GameOver; }
    */
    // We need to simulate the transition.
    // This logic is in `advance_phase`.
    // We can simulate it by being in Execution with AP=0 and calling VoteReady? No.
    // `VoteReady` calls `advance_phase`.

    // Let's set up state in `TacticalPlanning`.
    state.phase = GamePhase::TacticalPlanning;
    state.hull_integrity = 1;
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 0;
    }
    if let Some(r) = state.map.rooms.get_mut(&kitchen) {
        r.hazards.push(HazardType::Fire);
    }

    // Advance P -> E
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();
    // Advance E -> EA -> GameOver
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    assert_eq!(state.phase, GamePhase::GameOver);
}

#[test]
fn test_game_over_crew() {
    let mut state = GameLogic::new_game(vec!["P1".to_string(), "P2".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Kill crew
    if let Some(p) = state.players.get_mut("P1") {
        p.status.push(PlayerStatus::Fainted);
        p.ap = 0;
    }
    if let Some(p) = state.players.get_mut("P2") {
        p.status.push(PlayerStatus::Fainted);
        p.ap = 0;
    }

    // Advance P -> E
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();
    state = GameLogic::apply_action(
        state,
        "P2",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    // Advance E -> EA -> GameOver
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();
    // P2 ready needed? `advance_phase` logic: "if any AP left... else resolve...".
    // If we call VoteReady for P1, and AP=0, it triggers resolution?
    // Wait, `advance_phase` checks `state.players.values().all(|p| p.is_ready)`.
    // So both must be ready.
    state = GameLogic::apply_action(
        state,
        "P2",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    assert_eq!(state.phase, GamePhase::GameOver);
}

#[test]
fn test_join_mid_game() {
    let state = GameLogic::new_game(vec!["P1".to_string()], 12345);

    // Join P2
    let res = GameLogic::apply_action(
        state.clone(),
        "P2",
        Action::Meta(MetaAction::Join {
            name: "P2".to_string(),
        }),
        None,
    );
    assert!(res.is_ok());
    let state = res.unwrap();

    assert!(state.players.contains_key("P2"));
}

#[test]
fn test_full_sync_import() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.turn_count = 100;

    let json = serde_json::to_string(&state).unwrap();

    // New blank state
    let empty = GameLogic::new_game(vec![], 0);

    let res = GameLogic::apply_action(
        empty,
        "Any",
        Action::Meta(MetaAction::FullSync { state_json: json }),
        None,
    );
    assert!(res.is_ok());
    let synced = res.unwrap();

    assert_eq!(synced.turn_count, 100);
}
