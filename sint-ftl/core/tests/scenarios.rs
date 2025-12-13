use sint_core::{
    GameLogic,
    types::{Action, GameAction, GamePhase, ItemType},
};

#[test]
fn test_scenario_fire_in_kitchen() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned(), "P2".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let kitchen = sint_core::logic::find_room_with_system_in_map(
        &state.map,
        sint_core::types::SystemType::Kitchen,
    )
    .unwrap();
    let hallway = 0; // Hub

    if let Some(r) = state.map.rooms.get_mut(&kitchen) {
        r.hazards.push(sint_core::types::HazardType::Fire);
    }
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = hallway;
        p.ap = 2;
    }
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = kitchen;
        p.ap = 2;
    }

    // P2 Bake -> Fail
    let res = GameLogic::apply_action(state.clone(), "P2", Action::Game(GameAction::Bake), None);
    assert!(res.is_err());

    // P2 Extinguish -> Success (Queue)
    let mut state =
        GameLogic::apply_action(state, "P2", Action::Game(GameAction::Extinguish), None).unwrap();

    // P1 Move Hallway -> Kitchen
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::Move { to_room: kitchen }),
        None,
    )
    .unwrap();

    // P1 Bake
    // Note: In projection, fire is gone (P2 extinguished).
    state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::Bake), None).unwrap();

    // Execute
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

    // Check results
    assert!(state.map.rooms[&kitchen].hazards.is_empty());
    // P1 baked -> 3 nuts.
    // P1 picked up? No, items stay in room.
    assert!(
        state.map.rooms[&kitchen]
            .items
            .contains(&ItemType::Peppernut)
    );
}

#[test]
fn test_scenario_bucket_brigade() {
    let mut state = GameLogic::new_game(
        vec!["P1".to_owned(), "P2".to_owned(), "P3".to_owned()],
        12345,
    );
    state.phase = GamePhase::TacticalPlanning;

    let kitchen = sint_core::logic::find_room_with_system_in_map(
        &state.map,
        sint_core::types::SystemType::Kitchen,
    )
    .unwrap();
    let cannons = sint_core::logic::find_room_with_system_in_map(
        &state.map,
        sint_core::types::SystemType::Cannons,
    )
    .unwrap();
    let hallway = 0; // Hub

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = kitchen;
        p.ap = 2;
    }
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = hallway;
        p.ap = 2;
    }
    if let Some(p) = state.players.get_mut("P3") {
        p.room_id = cannons;
        p.ap = 2;
    }

    // P1
    state =
        GameLogic::apply_action(state.clone(), "P1", Action::Game(GameAction::Bake), None).unwrap();
    // Pickup baked item
    state = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::PickUp {
            item_type: ItemType::Peppernut,
        }),
        None,
    )
    .unwrap();

    // Cheat AP for P1 to allow Throw
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 10;
    }

    state = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Throw {
            target_player: "P2".to_owned(),
            item_index: 0,
        }),
        None,
    )
    .unwrap();

    // P2 has Nut now (in projection).
    // P2 Throws to P3.
    state = GameLogic::apply_action(
        state.clone(),
        "P2",
        Action::Game(GameAction::Throw {
            target_player: "P3".to_owned(),
            item_index: 0,
        }),
        None,
    )
    .unwrap();

    // P3 Shoots.
    state = GameLogic::apply_action(state.clone(), "P3", Action::Game(GameAction::Shoot), None)
        .unwrap();

    // Execute
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
        "P3",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    // Enemy HP damage?
    // Assuming hit.
    assert!(state.enemy.hp < 50); // Default boss
}
