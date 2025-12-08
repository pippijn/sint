use sint_core::{Action, GameLogic, GamePhase, ItemType};

#[test]
fn test_pickup_resolution() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);

    // Cheat phase to Planning
    state.phase = GamePhase::TacticalPlanning;

    // Setup Room 3 with specific items
    if let Some(room) = state.map.rooms.get_mut(&3) {
        room.items = vec![
            ItemType::Peppernut,
            ItemType::Extinguisher,
            ItemType::Peppernut,
        ];
    }

    // 1. Queue PickUp(Extinguisher)
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::PickUp {
            item_type: ItemType::Extinguisher,
        },
        None,
    )
    .unwrap();

    // Assert Queued
    assert_eq!(state.proposal_queue.len(), 1);

    // 2. Advance Phase to Execution (VoteReady)
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

    // Since "VoteReady" checks consensus, and P1 is the only player,
    // it should advance to Execution and RESOLVE the queue.

    assert_eq!(state.phase, GamePhase::Execution);

    // NOW check results
    let room = state.map.rooms.get(&3).unwrap();
    assert_eq!(room.items.len(), 2);
    // Extinguisher was in the middle, so removal should leave two Peppernuts
    assert_eq!(room.items[0], ItemType::Peppernut);
    assert_eq!(room.items[1], ItemType::Peppernut);

    let p = state.players.get("P1").unwrap();
    assert_eq!(p.inventory.len(), 1);
    assert_eq!(p.inventory[0], ItemType::Extinguisher);
}
