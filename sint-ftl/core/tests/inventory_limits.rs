use sint_core::{
    types::{Action, GameAction, GamePhase, ItemType},
    GameLogic,
};

#[test]
fn test_cannot_carry_two_special_items() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let room_id = state.players["P1"].room_id;

    // Give P1 an Extinguisher
    if let Some(p) = state.players.get_mut("P1") {
        p.inventory.push(ItemType::Extinguisher);
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
    assert!(res.is_err(), "Should not allow picking up 2nd special item");
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
        p.inventory.push(ItemType::Wheelbarrow);
    }
    if let Some(p) = state.players.get_mut("P1") {
        p.inventory.push(ItemType::Extinguisher); // Index 0
    }

    // P1 throws Extinguisher to P2 (who has Wheelbarrow)
    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Throw {
            target_player: "P2".to_owned(),
            item_index: 0,
        }),
        None,
    );
    assert!(res.is_err(), "Receiver cannot hold 2 special items");
}

#[test]
fn test_throw_fails_if_receiver_full_ammo() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned(), "P2".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Place P1 and P2 in same room
    let room_id = state.players["P1"].room_id;
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = room_id;
        p.inventory.push(ItemType::Peppernut); // Has 1 (Max 1)
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
