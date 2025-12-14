use sint_core::{
    GameLogic,
    logic::{find_room_with_system_in_map, resolution},
    types::*,
};

#[test]
fn test_system_explosion_damages_hull() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 42);
    state.phase = GamePhase::EnemyAction;
    state.hull_integrity = 20;

    let kitchen_id = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    if let Some(room) = state.map.rooms.get_mut(&kitchen_id) {
        // System health is 3. 3 Fires should break it immediately.
        room.hazards
            .extend_from_slice(&[HazardType::Fire, HazardType::Fire, HazardType::Fire]);
    }

    resolution::resolve_hazards(&mut state);

    assert_eq!(state.map.rooms[&kitchen_id].system_health, 0);
    assert!(state.map.rooms[&kitchen_id].is_broken);
    assert_eq!(
        state.hull_integrity, 19,
        "Hull should take 1 damage when a system explodes"
    );
}

#[test]
fn test_peppernut_destroyed_on_drop_into_water() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 42);
    state.phase = GamePhase::TacticalPlanning;

    let kitchen_id = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    // Add Water to room
    state
        .map
        .rooms
        .get_mut(&kitchen_id)
        .unwrap()
        .add_hazard(HazardType::Water);

    // Give player a nut
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = kitchen_id;
        p.inventory.push(ItemType::Peppernut);
    }

    // Action: Drop the nut (index 0)
    let state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::Drop { item_index: 0 }),
        None,
    )
    .unwrap();

    // Check room items
    let room = state.map.rooms.get(&kitchen_id).unwrap();
    assert!(
        !room.items.contains(&ItemType::Peppernut),
        "Peppernut should be destroyed immediately when dropped into Water"
    );
}

#[test]
fn test_wheelbarrow_drop_restriction() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 42);
    state.phase = GamePhase::TacticalPlanning;

    // Give player a Wheelbarrow and 3 nuts (allowed)
    if let Some(p) = state.players.get_mut("P1") {
        p.inventory.push(ItemType::Wheelbarrow);
        p.inventory.push(ItemType::Peppernut);
        p.inventory.push(ItemType::Peppernut);
        p.inventory.push(ItemType::Peppernut);
    }

    // Attempt to drop Wheelbarrow (index 0)
    let res = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::Drop { item_index: 0 }),
        None,
    );

    assert!(
        res.is_err(),
        "Should not be able to drop Wheelbarrow while holding more than 2 Peppernuts"
    );
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
