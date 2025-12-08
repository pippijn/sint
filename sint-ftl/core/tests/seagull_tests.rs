use sint_core::{Action, CardId, GameLogic, GamePhase, ItemType};

#[test]
fn test_seagull_attack_validation_in_planning() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Inject Seagull Attack
    let seagull_card = state
        .deck
        .iter()
        .find(|c| c.id == CardId::SeagullAttack)
        .cloned()
        .unwrap();
    state.active_situations.push(seagull_card);

    // Give P1 a Peppernut
    if let Some(p) = state.players.get_mut("P1") {
        p.inventory.push(ItemType::Peppernut);
        p.room_id = 7; // Hallway
    }

    // Attempt Move
    let res = GameLogic::apply_action(state, "P1", Action::Move { to_room: 6 }, None);

    assert!(
        res.is_err(),
        "Should be blocked by Seagull Attack validation"
    );
    if let Err(e) = res {
        assert!(
            e.to_string().contains("Seagull Attack"),
            "Error should mention Seagull Attack"
        );
    }
}

#[test]
fn test_seagull_attack_prevents_move_after_pickup() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Inject
    let seagull_card = state
        .deck
        .iter()
        .find(|c| c.id == CardId::SeagullAttack)
        .cloned()
        .unwrap();
    state.active_situations.push(seagull_card);

    // P1 in Room 11 (Storage, has nuts)
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 11;
        p.ap = 2;
    }

    // 1. Queue PickUp (Cost 1 AP)
    let state = GameLogic::apply_action(
        state,
        "P1",
        Action::PickUp {
            item_type: ItemType::Peppernut,
        },
        None,
    )
    .unwrap();

    // 2. Queue Move (Cost 1 AP)
    // This should now FAIL because projected state has Peppernut
    let res = GameLogic::apply_action(state, "P1", Action::Move { to_room: 7 }, None);

    assert!(
        res.is_err(),
        "Move should be blocked by Seagull Attack (Fail Early)"
    );
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("Cannot move while holding Peppernuts"));
}
