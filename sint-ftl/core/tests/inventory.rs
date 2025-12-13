use sint_core::{
    GameLogic,
    types::{Action, GameAction, GamePhase, ItemType},
};

#[test]
fn test_pickup_limit() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let kitchen = sint_core::logic::find_room_with_system_in_map(
        &state.map,
        sint_core::types::SystemType::Kitchen,
    )
    .unwrap();

    // Give P1 a Nut
    if let Some(p) = state.players.get_mut("P1") {
        p.inventory.push(ItemType::Peppernut);
        p.room_id = kitchen;
    }
    // Room has Nuts (from Bake or setup). Let's add one.
    if let Some(r) = state.map.rooms.get_mut(&kitchen) {
        r.items.push(ItemType::Peppernut);
    }

    // Try to pick up another Nut (Limit 1 without Wheelbarrow)
    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::PickUp {
            item_type: ItemType::Peppernut,
        }),
        None,
    );
    assert!(res.is_err(), "Should enforce 1 Nut limit");
}

#[test]
fn test_pickup_limit_with_wheelbarrow() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let kitchen = sint_core::logic::find_room_with_system_in_map(
        &state.map,
        sint_core::types::SystemType::Kitchen,
    )
    .unwrap();

    if let Some(p) = state.players.get_mut("P1") {
        p.inventory.push(ItemType::Wheelbarrow);
        p.inventory.push(ItemType::Peppernut);
        p.room_id = kitchen;
    }
    if let Some(r) = state.map.rooms.get_mut(&kitchen) {
        r.items.push(ItemType::Peppernut);
    }

    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::PickUp {
            item_type: ItemType::Peppernut,
        }),
        None,
    );
    assert!(res.is_ok(), "Wheelbarrow allows 5 Nuts");
}

#[test]
fn test_drop_wheelbarrow_fail_if_full() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    if let Some(p) = state.players.get_mut("P1") {
        p.inventory.push(ItemType::Wheelbarrow); // Index 0
        p.inventory.push(ItemType::Peppernut);
        p.inventory.push(ItemType::Peppernut);
    }

    // Try to drop Wheelbarrow (Index 0)
    let res = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::Drop { item_index: 0 }),
        None,
    );
    assert!(res.is_err(), "Cannot drop Wheelbarrow if holding >1 Nut");
}

#[test]
fn test_throw_item() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned(), "P2".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let kitchen = sint_core::logic::find_room_with_system_in_map(
        &state.map,
        sint_core::types::SystemType::Kitchen,
    )
    .unwrap();
    let hallway = 0;

    if let Some(p) = state.players.get_mut("P1") {
        p.inventory.push(ItemType::Peppernut); // Index 0
        p.room_id = kitchen;
    }
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = hallway; // Neighbor of Kitchen in Star Layout
    }

    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Throw {
            target_player: "P2".to_owned(),
            item_index: 0,
        }),
        None,
    );
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
    state = GameLogic::apply_action(
        state,
        "P2",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    assert!(state.players["P1"].inventory.is_empty());
    assert_eq!(state.players["P2"].inventory[0], ItemType::Peppernut);
}
