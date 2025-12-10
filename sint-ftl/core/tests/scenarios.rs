use sint_core::{
    types::{Action, GameAction, GamePhase, ItemType},
    GameLogic,
};

#[test]
fn test_scenario_fire_in_kitchen() {
    let mut state = GameLogic::new_game(vec!["P1".to_string(), "P2".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Scenario: Fire in Kitchen (6). P1 in Hallway (7). P2 in Kitchen.
    // P2 tries to Bake (Fail).
    // P2 Extinguish (Success).
    // P1 moves to Kitchen.
    // P1 Bake (Success).

    if let Some(r) = state.map.rooms.get_mut(&6) {
        r.hazards.push(sint_core::types::HazardType::Fire);
    }
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 7;
        p.ap = 2;
    }
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = 6;
        p.ap = 2;
    }

    // P2 Bake -> Fail
    let res = GameLogic::apply_action(state.clone(), "P2", Action::Game(GameAction::Bake), None);
    assert!(res.is_err());

    // P2 Extinguish -> Success (Queue)
    let mut state =
        GameLogic::apply_action(state, "P2", Action::Game(GameAction::Extinguish), None).unwrap();

    // P1 Move 7 -> 6
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::Move { to_room: 6 }),
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
    assert!(state.map.rooms[&6].hazards.is_empty());
    // P1 baked -> 3 nuts.
    // P1 picked up? No, items stay in room.
    assert!(state.map.rooms[&6].items.contains(&ItemType::Peppernut));
}

#[test]
fn test_scenario_bucket_brigade() {
    let mut state = GameLogic::new_game(
        vec!["P1".to_string(), "P2".to_string(), "P3".to_string()],
        12345,
    );
    state.phase = GamePhase::TacticalPlanning;

    // Chain: P1 (Kitchen 6) -> P2 (Hallway 7) -> P3 (Cannons 8).
    // P1 Bakes. P1 Throws to P2. P2 Throws to P3. P3 Shoots.

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 6;
        p.ap = 2;
    }
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = 7;
        p.ap = 2;
    }
    if let Some(p) = state.players.get_mut("P3") {
        p.room_id = 8;
        p.ap = 2;
    }

    // P1
    state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::Bake), None).unwrap();
    // Need to pickup first? Bake spawns on floor.
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::PickUp {
            item_type: ItemType::Peppernut,
        }),
        None,
    )
    .unwrap();
    // Oops, P1 out of AP (Bake=1, Pickup=1). Cannot throw.
    // P1 needs 3 AP for this chain alone? Or Bake happens before?
    // Let's assume P1 started with a Nut or has 3 AP (TurboMode?).
    // Or P1 only Bakes and Drops? No, Throw.

    // Let's cheat AP for P1.
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 10;
    }

    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::Throw {
            target_player: "P2".to_string(),
            item_index: 0,
        }),
        None,
    )
    .unwrap();

    // P2 has Nut now (in projection).
    // P2 Throws to P3.
    state = GameLogic::apply_action(
        state,
        "P2",
        Action::Game(GameAction::Throw {
            target_player: "P3".to_string(),
            item_index: 0,
        }),
        None,
    )
    .unwrap();

    // P3 Shoots.
    state = GameLogic::apply_action(state, "P3", Action::Game(GameAction::Shoot), None).unwrap();

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
