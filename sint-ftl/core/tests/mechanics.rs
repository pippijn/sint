use sint_core::{
    GameLogic,
    logic::{find_room_with_system_in_map, resolution},
    types::*,
};

#[test]
fn test_cannon_hit() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let cannons = find_room_with_system_in_map(&state.map, SystemType::Cannons).unwrap();

    // Setup: P1 in Cannons, Has Nut. Enemy HP 5.
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = cannons;
        p.inventory.push(ItemType::Peppernut);
    }

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

    assert!(state.enemy.hp < 5);
}

#[test]
fn test_shields_block_damage() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let bridge = find_room_with_system_in_map(&state.map, SystemType::Bridge).unwrap();
    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    // P1 in Bridge
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = bridge;
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
    state.enemy.next_attack = Some(EnemyAttack {
        target_room: Some(kitchen),
        target_system: Some(SystemType::Kitchen),
        effect: AttackEffect::Fireball,
    });

    resolution::resolve_enemy_attack(&mut state);

    assert_eq!(state.hull_integrity, 20);
    assert!(state.map.rooms[&kitchen].hazards.is_empty());
}

#[test]
fn test_evasion_blocks_hit() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let engine = find_room_with_system_in_map(&state.map, SystemType::Engine).unwrap();
    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    // P1 in Engine
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = engine;
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
    state.enemy.next_attack = Some(EnemyAttack {
        target_room: Some(kitchen),
        target_system: Some(SystemType::Kitchen),
        effect: AttackEffect::Fireball,
    });

    resolution::resolve_enemy_attack(&mut state);

    assert_eq!(state.hull_integrity, 20);
    assert!(state.map.rooms[&kitchen].hazards.is_empty());
}

#[test]
fn test_leak_deals_hull_damage() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 42);
    state.phase = GamePhase::EnemyAction;
    state.hull_integrity = 20;

    let kitchen_id = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    state.enemy.next_attack = Some(EnemyAttack {
        target_room: Some(kitchen_id),
        target_system: Some(SystemType::Kitchen),
        effect: AttackEffect::Leak,
    });

    resolution::resolve_enemy_attack(&mut state);

    assert_eq!(state.hull_integrity, 19);
    assert!(
        state.map.rooms[&kitchen_id]
            .hazards
            .contains(&HazardType::Water)
    );
}

#[test]
fn test_fireball_deals_hull_damage() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 42);
    state.phase = GamePhase::EnemyAction;
    state.hull_integrity = 20;

    let kitchen_id = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    state.enemy.next_attack = Some(EnemyAttack {
        target_room: Some(kitchen_id),
        target_system: Some(SystemType::Kitchen),
        effect: AttackEffect::Fireball,
    });

    resolution::resolve_enemy_attack(&mut state);

    assert_eq!(state.hull_integrity, 19);
    assert!(
        state.map.rooms[&kitchen_id]
            .hazards
            .contains(&HazardType::Fire)
    );
}

#[test]
fn test_boss_progression() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let cannons = find_room_with_system_in_map(&state.map, SystemType::Cannons).unwrap();

    state.enemy.hp = 1;
    state.boss_level = 0;

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = cannons;
        p.inventory.push(ItemType::Peppernut);
        p.ap = 2;
    }

    let res = GameLogic::apply_action(state.clone(), "P1", Action::Game(GameAction::Shoot), None);
    let mut state = res.unwrap();

    // Force AP to 0 to advance phase on VoteReady
    state.players.get_mut("P1").unwrap().ap = 0;

    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    // Now in Execution. VoteReady again with 0 AP to reach EnemyAction.
    state.players.get_mut("P1").unwrap().is_ready = false;
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    assert_eq!(state.phase, GamePhase::EnemyAction);
    assert_eq!(state.enemy.state, EnemyState::Defeated);

    state.players.get_mut("P1").unwrap().is_ready = false;
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();
    assert_eq!(state.phase, GamePhase::MorningReport);
    assert!(state.is_resting);

    state.players.get_mut("P1").unwrap().is_ready = false;
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap(); // Morning -> Telegraph
    state.players.get_mut("P1").unwrap().is_ready = false;
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap(); // Telegraph -> Tactical

    assert_eq!(state.phase, GamePhase::TacticalPlanning);

    state.players.get_mut("P1").unwrap().ap = 0;
    state.players.get_mut("P1").unwrap().is_ready = false;
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    assert_eq!(state.phase, GamePhase::Execution);

    state.players.get_mut("P1").unwrap().is_ready = false;
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    assert_eq!(state.phase, GamePhase::EnemyAction);
    assert!(state.is_resting);

    state.players.get_mut("P1").unwrap().is_ready = false;
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    assert_eq!(state.boss_level, 1);
    assert_eq!(state.enemy.name, "The Monster");
    assert!(!state.is_resting);
}

#[test]
fn test_game_over_hull() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    state.phase = GamePhase::TacticalPlanning;
    state.hull_integrity = 1;
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 0;
    }
    if let Some(r) = state.map.rooms.get_mut(&kitchen) {
        r.hazards.push(HazardType::Fire);
        r.system_health = 1; // Ensure explosion deals hull damage
    }

    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();
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
    let mut state = GameLogic::new_game(vec!["P1".to_owned(), "P2".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    if let Some(p) = state.players.get_mut("P1") {
        p.status.push(PlayerStatus::Fainted);
        p.ap = 0;
    }
    if let Some(p) = state.players.get_mut("P2") {
        p.status.push(PlayerStatus::Fainted);
        p.ap = 0;
    }

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

    assert_eq!(state.phase, GamePhase::GameOver);
}

#[test]
fn test_join_mid_game() {
    let state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    let res = GameLogic::apply_action(
        state.clone(),
        "P2",
        Action::Meta(MetaAction::Join {
            name: "P2".to_owned(),
        }),
        None,
    );
    assert!(res.is_ok());
    let state = res.unwrap();
    assert!(state.players.contains_key("P2"));
}

#[test]
fn test_full_sync_import() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.turn_count = 100;
    let json = serde_json::to_string(&state).unwrap();
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

#[test]
fn test_fainted_player_cannot_act() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;
    if let Some(p) = state.players.get_mut("P1") {
        p.status.push(PlayerStatus::Fainted);
    }
    let res = GameLogic::apply_action(state.clone(), "P1", Action::Game(GameAction::Pass), None);
    assert!(res.is_err());
    let actions = GameLogic::get_valid_actions(&state, "P1");
    let has_move = actions
        .iter()
        .any(|a| matches!(a, Action::Game(GameAction::Move { .. })));
    assert!(!has_move);
}

#[test]
fn test_revive_mechanics() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned(), "P2".to_owned()], 42);
    state.phase = GamePhase::TacticalPlanning;

    let kitchen_id = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    // Setup: P2 is Fainted in Kitchen. P1 is in Kitchen.
    if let Some(p2) = state.players.get_mut("P2") {
        p2.hp = 0;
        p2.status.push(PlayerStatus::Fainted);
        p2.room_id = kitchen_id;
    }
    if let Some(p1) = state.players.get_mut("P1") {
        p1.room_id = kitchen_id;
    }

    // Action: P1 revives P2
    let state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::Revive {
            target_player: "P2".to_owned(),
        }),
        None,
    )
    .unwrap();

    // Resolve (Queue)
    let mut state = state;
    resolution::resolve_proposal_queue(&mut state, false);

    let p2 = state.players.get("P2").unwrap();
    assert_eq!(p2.hp, 1, "Revived player should have 1 HP");
    assert!(
        !p2.status.contains(&PlayerStatus::Fainted),
        "Revived player should not have Fainted status"
    );
}
