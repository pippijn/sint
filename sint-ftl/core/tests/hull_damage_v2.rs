use sint_core::{
    GameLogic,
    logic::{find_room_with_system_in_map, resolution},
    types::*,
};

#[test]
fn test_leak_deals_hull_damage() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 42);
    state.phase = GamePhase::EnemyAction;
    state.hull_integrity = 20;

    let kitchen_id = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    // Set up a Leak attack
    state.enemy.next_attack = Some(EnemyAttack {
        target_room: Some(kitchen_id),
        target_system: Some(SystemType::Kitchen),
        effect: AttackEffect::Leak,
    });

    resolution::resolve_enemy_attack(&mut state);

    assert_eq!(
        state.hull_integrity, 19,
        "Leak (Direct Hit) should deal 1 hull damage"
    );
    assert!(
        state.map.rooms[&kitchen_id]
            .hazards
            .contains(&HazardType::Water),
        "Leak should add Water hazard"
    );
}

#[test]
fn test_fireball_deals_hull_damage() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 42);
    state.phase = GamePhase::EnemyAction;
    state.hull_integrity = 20;

    let kitchen_id = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    // Set up a Fireball attack
    state.enemy.next_attack = Some(EnemyAttack {
        target_room: Some(kitchen_id),
        target_system: Some(SystemType::Kitchen),
        effect: AttackEffect::Fireball,
    });

    resolution::resolve_enemy_attack(&mut state);

    assert_eq!(
        state.hull_integrity, 19,
        "Fireball (Direct Hit) should deal 1 hull damage"
    );
    assert!(
        state.map.rooms[&kitchen_id]
            .hazards
            .contains(&HazardType::Fire),
        "Fireball should add Fire hazard"
    );
}

#[test]
fn test_fire_does_not_deal_hull_damage_per_round() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 42);
    state.phase = GamePhase::EnemyAction;
    state.hull_integrity = 20;

    let kitchen_id = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    // Add Fire to room
    state
        .map
        .rooms
        .get_mut(&kitchen_id)
        .unwrap()
        .add_hazard(HazardType::Fire);

    resolution::resolve_hazards(&mut state);

    assert_eq!(
        state.hull_integrity, 20,
        "Fire should NOT deal hull damage per round (The Burn only damages players)"
    );
}

#[test]
fn test_fire_disables_system_action() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 42);
    state.phase = GamePhase::TacticalPlanning;

    let kitchen_id = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    // Move player to kitchen
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = kitchen_id;
    }

    // Add Fire to kitchen
    state
        .map
        .rooms
        .get_mut(&kitchen_id)
        .unwrap()
        .add_hazard(HazardType::Fire);

    let actions = GameLogic::get_valid_actions(&state, "P1");
    let has_bake = actions
        .iter()
        .any(|a| matches!(a, Action::Game(GameAction::Bake)));

    assert!(
        !has_bake,
        "System actions should be disabled if room has Fire"
    );
}
