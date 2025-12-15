use sint_core::{
    GameLogic,
    types::{Action, CardId, GameAction, GamePhase, GameState},
};

fn new_test_game(players: Vec<String>) -> GameState {
    let mut state = GameLogic::new_game(players, 12345);
    state.phase = GamePhase::TacticalPlanning;
    state
}

#[test]
fn test_sugar_rush_teleport() {
    let mut state = new_test_game(vec!["P1".to_owned()]);

    // Move P1 to room 1
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 1;
        p.ap = 1; // ONLY 1 AP
    }

    // Room 1 and 2 are not adjacent in Star layout (path is 1 -> 0 -> 2, length 2)
    let move_to_2 = Action::Game(GameAction::Move { to_room: 2 });

    // 1. Without Sugar Rush, move from 1 to 2 should fail because it costs 2 AP
    let result = GameLogic::apply_action(state.clone(), "P1", move_to_2.clone(), None);
    assert!(
        result.is_err(),
        "Move to non-adjacent room should fail without Sugar Rush due to cost (2 AP > 1 AP available)"
    );

    // 2. With Sugar Rush, move from 1 to 2 should succeed and cost only 1 AP
    use sint_core::logic::cards::get_behavior;
    let behavior = get_behavior(CardId::SugarRush);
    let card = behavior.get_struct();
    state.active_situations.push(card);

    let result = GameLogic::apply_action(state.clone(), "P1", move_to_2.clone(), None);
    assert!(
        result.is_ok(),
        "Move to non-adjacent room should succeed with Sugar Rush: {:?}",
        result.err()
    );

    let state_after_move = result.unwrap();
    // In apply_action, the action is queued. We need to check the proposal queue.
    assert_eq!(state_after_move.proposal_queue.len(), 1);
    assert!(matches!(
        state_after_move.proposal_queue[0].action,
        GameAction::Move { to_room: 2 }
    ));

    // Check AP cost (should be 1 AP total for the leap)
    let p_after = state_after_move.players.get("P1").unwrap();
    assert_eq!(
        p_after.ap, 0,
        "Leaping move should cost exactly 1 AP (1 - 1 = 0)"
    );

    // 3. Verify get_valid_actions returns all rooms
    let valid_actions = GameLogic::get_valid_actions(&state, "P1");
    let move_targets: Vec<u32> = valid_actions
        .iter()
        .filter_map(|a| {
            if let Action::Game(GameAction::Move { to_room }) = a {
                Some(*to_room)
            } else {
                None
            }
        })
        .collect();

    // There are 10 rooms (0-9). From room 1, room 1 itself is not a valid move usually.
    // Neighbors are just [0].
    // With Sugar Rush, all OTHER rooms should be reachable.
    for i in 0..10 {
        if i == 1 {
            continue;
        }
        assert!(
            move_targets.contains(&i),
            "Room {} should be a valid move target with Sugar Rush",
            i
        );
    }
}
