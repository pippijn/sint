use sint_core::{
    GameError,
    logic::{GameLogic, cards::get_behavior, resolution},
    types::*,
};

fn create_test_state() -> GameState {
    let players = vec!["P1".to_owned(), "P2".to_owned()];
    GameLogic::new_game(players, 12345)
}

#[test]
fn test_recipe_reward() {
    let mut state = create_test_state();
    let behavior = get_behavior(CardId::Recipe);

    // Setup: P1 empty inventory
    let p1 = state.players.get_mut("P1").unwrap();
    p1.inventory.clear();

    // Act: Solve
    behavior.on_solved(&mut state);

    // Assert: P1 has 2 Peppernuts
    let p1 = state.players.get("P1").unwrap();
    let nut_count = p1
        .inventory
        .iter()
        .filter(|i| **i == ItemType::Peppernut)
        .count();
    assert_eq!(nut_count, 2, "Recipe should grant 2 Peppernuts");
}

#[test]
fn test_staff_reward() {
    let mut state = create_test_state();
    let behavior = get_behavior(CardId::TheStaff);

    // Setup: P1 damaged and fainted
    let p1 = state.players.get_mut("P1").unwrap();
    p1.hp = 1;
    p1.status.push(PlayerStatus::Fainted);

    // Act: Solve
    behavior.on_solved(&mut state);

    // Assert: Full HP and No Fainted
    let p1 = state.players.get("P1").unwrap();
    assert_eq!(p1.hp, 3, "Staff should heal to full");
    assert!(
        !p1.status.contains(&PlayerStatus::Fainted),
        "Staff should remove Fainted"
    );
}

#[test]
fn test_golden_nut_reward() {
    let mut state = create_test_state();
    let behavior = get_behavior(CardId::GoldenNut);

    // Setup: Enemy HP
    state.enemy.hp = 5;

    // Act: Solve
    behavior.on_solved(&mut state);

    // Assert: Enemy takes 1 damage
    assert_eq!(state.enemy.hp, 4, "Golden Nut should deal 1 damage");
}

#[test]
fn test_book_reward() {
    let mut state = create_test_state();
    let behavior = get_behavior(CardId::TheBook);

    // Setup: Enemy attack queued
    state.enemy.next_attack = Some(EnemyAttack {
        target_room: Some(5),
        target_system: Some(sint_core::types::SystemType::Engine),
        effect: AttackEffect::Fireball,
    });

    // Act: Solve
    behavior.on_solved(&mut state);

    // Assert: Attack cancelled
    assert!(
        state.enemy.next_attack.is_none(),
        "The Book should cancel enemy attack"
    );
}

#[test]
fn test_rudderless_hazard_modifier() {
    let mut state = create_test_state();

    // Add Rudderless card
    let card = get_behavior(CardId::Rudderless).get_struct();
    state.active_situations.push(card);

    state.enemy.next_attack = Some(EnemyAttack {
        target_room: Some(5),
        target_system: Some(sint_core::types::SystemType::Engine),
        effect: AttackEffect::Fireball,
    });

    // Act: Resolve Attack
    resolution::resolve_enemy_attack(&mut state);

    // Assert: Room 5 should have 1 (Base) + 1 (Modifier) = 2 Fire tokens
    let room = state.map.rooms.get(&5).unwrap();
    let fire_count = room
        .hazards
        .iter()
        .filter(|h| **h == HazardType::Fire)
        .count();
    // Note: Hull damage happens once per hit, but hazard spawn is modified.
    // Base implementation: "room.hazards.push(HazardType::Fire);" then loop modifier.
    assert_eq!(fire_count, 2, "Rudderless should add +1 Fire token");
}

#[test]
fn test_wailing_alarm_blocks() {
    let state = create_test_state();
    let behavior = get_behavior(CardId::WailingAlarm);

    // Test Shield Block
    let action = GameAction::RaiseShields;
    let res = behavior.validate_action(&state, "P1", &action);
    assert!(
        matches!(res, Err(GameError::InvalidAction(_))),
        "Alarm should block Shields"
    );

    // Test Evasion Block
    let action = GameAction::EvasiveManeuvers;
    let res = behavior.validate_action(&state, "P1", &action);
    assert!(
        matches!(res, Err(GameError::InvalidAction(_))),
        "Alarm should block Evasion"
    );
}

#[test]
fn test_fog_bank_masking() {
    let mut state = create_test_state();
    let behavior = get_behavior(CardId::FogBank);

    // Setup: Initial Attack
    let mut attack = EnemyAttack {
        target_room: Some(5),
        target_system: Some(sint_core::types::SystemType::Engine),
        effect: AttackEffect::Fireball,
    };

    // Act 1: Modify (Mask)
    behavior.modify_telegraph(&mut attack);

    assert_eq!(attack.target_room, None);
    assert!(matches!(attack.effect, AttackEffect::Hidden));

    // Act 2: Resolve (Reveal)
    behavior.resolve_telegraph(&mut state, &mut attack);

    assert!(attack.target_room.is_some(), "Target should be revealed");
    assert!(
        matches!(attack.effect, AttackEffect::Fireball),
        "Effect should revert to Fireball"
    );
}

#[test]
fn test_attack_wave() {
    let state = create_test_state();
    let behavior = get_behavior(CardId::AttackWave);

    assert_eq!(behavior.get_enemy_attack_count(&state), 2);
}

#[test]
fn test_big_leak() {
    let mut state = create_test_state();
    let behavior = get_behavior(CardId::BigLeak);
    let cargo_id = sint_core::logic::find_room_with_system(&state, SystemType::Cargo).unwrap();

    behavior.on_round_start(&mut state);

    let room = state.map.rooms.get(&cargo_id).unwrap();
    assert!(room.hazards.contains(&HazardType::Water));
}

#[test]
fn test_clogged_pipe() {
    let state = create_test_state();
    let behavior = get_behavior(CardId::CloggedPipe);

    let res = behavior.validate_action(&state, "P1", &GameAction::Bake);
    assert!(res.is_err());
}

#[test]
fn test_stowaway_trigger() {
    let mut state = create_test_state();
    let behavior = get_behavior(CardId::Stowaway);

    if let Some(p) = state.players.get_mut("P1") {
        p.inventory.push(ItemType::Peppernut);
    }

    behavior.on_trigger(&mut state);

    let p1 = state.players.get("P1").unwrap();
    assert!(p1.inventory.is_empty());
}

#[test]
fn test_falling_gift() {
    let mut state = create_test_state();
    let behavior = get_behavior(CardId::FallingGift);
    let cargo_id = sint_core::logic::find_room_with_system(&state, SystemType::Cargo).unwrap();

    behavior.on_activate(&mut state);

    let room = state.map.rooms.get(&cargo_id).unwrap();
    assert!(room.hazards.contains(&HazardType::Water));
    assert!(room.items.contains(&ItemType::Peppernut));
}

#[test]
fn test_high_pressure() {
    let mut state = create_test_state();
    let behavior = get_behavior(CardId::HighPressure);
    let initial_room = state.players["P1"].room_id;

    behavior.on_activate(&mut state);

    assert_ne!(state.players["P1"].room_id, initial_room);
}

#[test]
fn test_high_waves() {
    let mut state = create_test_state();
    let behavior = get_behavior(CardId::HighWaves);
    let engine_id = sint_core::logic::find_room_with_system(&state, SystemType::Engine).unwrap();

    // Place P1 in Bridge (Room 7 in Star)
    let bridge_id = sint_core::logic::find_room_with_system(&state, SystemType::Bridge).unwrap();
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = bridge_id;
    }

    // Calculate expected first step towards Engine
    let path = sint_core::logic::pathfinding::find_path(&state.map, bridge_id, engine_id).unwrap();
    let expected_room = path[0];

    behavior.on_activate(&mut state);

    assert_eq!(state.players["P1"].room_id, expected_room);
}

#[test]
fn test_jammed_cannon() {
    let state = create_test_state();
    let behavior = get_behavior(CardId::JammedCannon);

    let res = behavior.validate_action(&state, "P1", &GameAction::Shoot);
    assert!(res.is_err());
}

#[test]
fn test_leak() {
    let mut state = create_test_state();
    let behavior = get_behavior(CardId::Leak);
    let cargo_id = sint_core::logic::find_room_with_system(&state, SystemType::Cargo).unwrap();

    behavior.on_activate(&mut state);

    let room = state.map.rooms.get(&cargo_id).unwrap();
    assert!(room.hazards.contains(&HazardType::Water));
}
