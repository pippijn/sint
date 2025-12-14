use sint_core::{
    GameLogic,
    logic::find_room_with_system_in_map,
    types::{Action, GameAction, GamePhase, HazardType, ItemType, SystemType},
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

    // Give P1 2 Nuts
    if let Some(p) = state.players.get_mut("P1") {
        p.inventory.push(ItemType::Peppernut);
        p.inventory.push(ItemType::Peppernut);
        p.room_id = kitchen;
    }
    // Room has Nuts (from Bake or setup). Let's add one.
    if let Some(r) = state.map.rooms.get_mut(&kitchen) {
        r.items.push(ItemType::Peppernut);
    }

    // Try to pick up another Nut (Limit 2 without Wheelbarrow)
    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::PickUp {
            item_type: ItemType::Peppernut,
        }),
        None,
    );
    assert!(res.is_err(), "Should enforce 2 Nut limit");
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
        p.inventory.push(ItemType::Peppernut);
    }

    // Try to drop Wheelbarrow (Index 0)
    let res = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::Drop { item_index: 0 }),
        None,
    );
    assert!(res.is_err(), "Cannot drop Wheelbarrow if holding >2 Nuts");
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

#[test]
fn test_cannot_carry_two_special_items() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let room_id = state.players["P1"].room_id;

    // Give P1 an Extinguisher
    if let Some(p) = state.players.get_mut("P1") {
        p.inventory.push(ItemType::Extinguisher);
        p.inventory.push(ItemType::Keychain);
    }
    // Place Wheelbarrow in room
    if let Some(r) = state.map.rooms.get_mut(&room_id) {
        r.items.push(ItemType::Wheelbarrow);
    }

    // Try to pick up Wheelbarrow
    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::PickUp {
            item_type: ItemType::Wheelbarrow,
        }),
        None,
    );
    assert!(
        res.is_err(),
        "Should not allow picking up Wheelbarrow if hands are full"
    );
}

#[test]
fn test_special_items_dont_count_towards_ammo_limit() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;
    let room_id = state.players["P1"].room_id;

    // Give P1 an Extinguisher
    if let Some(p) = state.players.get_mut("P1") {
        p.inventory.push(ItemType::Extinguisher);
    }
    // Place Peppernut in room
    if let Some(r) = state.map.rooms.get_mut(&room_id) {
        r.items.push(ItemType::Peppernut);
    }

    // Try to pick up Peppernut (Should be allowed, 0 Peppernuts held)
    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::PickUp {
            item_type: ItemType::Peppernut,
        }),
        None,
    );
    assert!(res.is_ok(), "Special item should not block 1st Peppernut");
}

#[test]
fn test_throw_fails_if_receiver_has_special_item() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned(), "P2".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Place P1 and P2 in same room
    let room_id = state.players["P1"].room_id;
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = room_id;
        p.inventory.push(ItemType::Extinguisher);
        p.inventory.push(ItemType::Keychain);
    }
    if let Some(p) = state.players.get_mut("P1") {
        p.inventory.push(ItemType::Peppernut); // Index 0
    }

    // P1 throws Peppernut to P2 (who has both hands full)
    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Throw {
            target_player: "P2".to_owned(),
            item_index: 0,
        }),
        None,
    );
    assert!(
        res.is_err(),
        "Receiver cannot hold item if both hands are full"
    );
}

#[test]
fn test_throw_fails_if_receiver_full_ammo() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned(), "P2".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Place P1 and P2 in same room
    let room_id = state.players["P1"].room_id;
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = room_id;
        p.inventory.push(ItemType::Peppernut);
        p.inventory.push(ItemType::Peppernut); // Has 2 (Max 2)
    }
    if let Some(p) = state.players.get_mut("P1") {
        p.inventory.push(ItemType::Peppernut); // Index 0
    }

    // P1 throws Peppernut to P2
    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Throw {
            target_player: "P2".to_owned(),
            item_index: 0,
        }),
        None,
    );
    assert!(res.is_err(), "Receiver cannot exceed ammo limit via throw");
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
