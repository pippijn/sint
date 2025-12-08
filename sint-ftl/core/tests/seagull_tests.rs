use sint_core::{Action, CardId, GameLogic, GamePhase, ItemType};

#[test]
fn test_seagull_attack_validation_in_planning() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Inject C03 Seagull Attack
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
fn test_seagull_attack_refund_in_execution() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // P1 queues a Move (valid at planning time, hypothetically, if we didn't check active cards immediately)
    // But since we added immediate validation, we can't easily test resolution refund
    // UNLESS the card appears AFTER planning.
    // Scenario:
    // 1. Plan Move.
    // 2. Someone draws C03 (e.g. via Morning Report? No, that's before Planning).
    // Wait, Morning Report happens BEFORE Planning. So C03 is known.
    // But what if P1 picks up a nut THEN moves in the same batch?
    // Move 1: Pick Up (valid)
    // Move 2: Move (invalid because now holding nut)

    // Inject C03
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
    // 2. Queue Move (Cost 1 AP) -> This should pass apply_action because inventory is checked against CURRENT state (empty).

    let state = GameLogic::apply_action(
        state,
        "P1",
        Action::PickUp {
            item_type: ItemType::Peppernut,
        },
        None,
    )
    .unwrap();

    // At this point, P1 inventory in state is still EMPTY because apply_action just queues.
    // So apply_action for Move should succeed validation.
    let mut state =
        GameLogic::apply_action(state, "P1", Action::Move { to_room: 7 }, None).unwrap();

    assert_eq!(state.players["P1"].ap, 0); // Both paid for

    // Now Execute
    state.phase = GamePhase::Execution;
    sint_core::logic::resolution::resolve_proposal_queue(&mut state);

    // Verify:
    // 1. PickUp succeeded (Inventory has nut)
    assert!(state.players["P1"].inventory.contains(&ItemType::Peppernut));
    // 2. Move failed (Still in Room 11)
    assert_eq!(state.players["P1"].room_id, 11);
    // 3. AP Refunded for the failed move?
    // Initial: 2. PickUp: -1. Move: -1. Remaining: 0.
    // Refund: +1. Final: 1.
    assert_eq!(
        state.players["P1"].ap, 1,
        "Should have refunded 1 AP for failed move"
    );
}
