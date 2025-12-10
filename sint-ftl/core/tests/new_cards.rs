use sint_core::{
    logic::{cards::get_behavior, resolution, GameLogic},
    types::*,
    GameError,
};

fn create_test_state() -> GameState {
    let players = vec!["P1".to_string(), "P2".to_string()];
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
        target_room: 5,
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

    // Setup Attack on Room 5 (Engine)
    state.enemy.next_attack = Some(EnemyAttack {
        target_room: 5,
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
    let action = Action::RaiseShields;
    let res = behavior.validate_action(&state, "P1", &action);
    assert!(
        matches!(res, Err(GameError::InvalidAction(_))),
        "Alarm should block Shields"
    );

    // Test Evasion Block
    let action = Action::EvasiveManeuvers;
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
        target_room: 5,
        effect: AttackEffect::Fireball,
    };

    // Act 1: Modify (Mask)
    behavior.modify_telegraph(&mut attack);

    assert_eq!(attack.target_room, 0);
    assert!(matches!(attack.effect, AttackEffect::Hidden));

    // Act 2: Resolve (Reveal)
    behavior.resolve_telegraph(&mut state, &mut attack);

    assert_ne!(attack.target_room, 0, "Target should be revealed");
    assert!(
        matches!(attack.effect, AttackEffect::Fireball),
        "Effect should revert to Fireball"
    );
}
