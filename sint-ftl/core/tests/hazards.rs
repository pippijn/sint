use sint_core::{
    types::{Action, GameAction, GamePhase, HazardType, SystemType},
    GameLogic,
};

#[test]
fn test_fire_spread() {
    // Seed selected to ensure spread happens
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::EnemyAction;

    // Room 6 (Kitchen) has 2 Fires. Spreads to 7 (Hallway) with 50% chance.
    if let Some(r) = state.map.rooms.get_mut(&6) {
        r.hazards.push(HazardType::Fire);
        r.hazards.push(HazardType::Fire);
    }

    sint_core::logic::resolution::resolve_hazards(&mut state);

    // Check if spread to 7 (Hallway is neighbor of Kitchen)
    if let Some(r) = state.map.rooms.get(&7) {
        // With seed 12345, check result
        assert!(!r.hazards.is_empty(), "Fire should have spread to Hallway");
    }
}

#[test]
fn test_fire_damage_hull() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::EnemyAction;
    state.hull_integrity = 20;

    if let Some(r) = state.map.rooms.get_mut(&6) {
        r.hazards.push(HazardType::Fire);
    }

    sint_core::logic::resolution::resolve_hazards(&mut state);

    assert_eq!(state.hull_integrity, 19);
}

#[test]
fn test_fire_damage_player() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::EnemyAction;

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 6;
        p.hp = 3;
    }
    if let Some(r) = state.map.rooms.get_mut(&6) {
        r.hazards.push(HazardType::Fire);
    }

    sint_core::logic::resolution::resolve_hazards(&mut state);

    assert_eq!(state.players["P1"].hp, 2);
}

#[test]
fn test_water_destroys_peppernuts() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    if let Some(r) = state.map.rooms.get_mut(&6) {
        r.hazards.push(HazardType::Water);
        r.items.push(sint_core::types::ItemType::Peppernut);
        r.items.push(sint_core::types::ItemType::Extinguisher);
    }

    sint_core::logic::resolution::resolve_hazards(&mut state);

    let items = &state.map.rooms[&6].items;
    assert!(!items.contains(&sint_core::types::ItemType::Peppernut));
    assert!(items.contains(&sint_core::types::ItemType::Extinguisher));
}

#[test]
fn test_water_in_storage_safe() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    let storage_id = SystemType::Storage.as_u32();
    if let Some(r) = state.map.rooms.get_mut(&storage_id) {
        r.hazards.push(HazardType::Water);
        r.items.push(sint_core::types::ItemType::Peppernut);
    }

    sint_core::logic::resolution::resolve_hazards(&mut state);

    let items = &state.map.rooms[&storage_id].items;
    assert!(items.contains(&sint_core::types::ItemType::Peppernut));
}

#[test]
fn test_extinguish_action() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 6;
    }
    if let Some(r) = state.map.rooms.get_mut(&6) {
        r.hazards.push(HazardType::Fire);
    }

    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Extinguish),
        None,
    );
    assert!(res.is_ok());

    let mut state = res.unwrap();
    // Resolve (Queue)
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    assert!(state.map.rooms[&6].hazards.is_empty());
}

#[test]
fn test_repair_action() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 6;
    }
    if let Some(r) = state.map.rooms.get_mut(&6) {
        r.hazards.push(HazardType::Water);
    }

    let res = GameLogic::apply_action(state.clone(), "P1", Action::Game(GameAction::Repair), None);
    assert!(res.is_ok());

    let mut state = res.unwrap();
    // Resolve
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    assert!(state.map.rooms[&6].hazards.is_empty());
}
