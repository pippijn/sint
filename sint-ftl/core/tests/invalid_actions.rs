use sint_core::{
    logic::{GameError, GameLogic, apply_action},
    types::*,
};

#[test]
fn test_action_from_invalid_player_id() {
    // 1. Setup: Create a game and set it to the planning phase.
    let mut state = GameLogic::new_game(vec!["p1".to_owned()], 0);
    state.phase = GamePhase::TacticalPlanning;

    // 2. Action: Attempt to perform an action with a player_id that does not exist.
    let move_action = Action::Game(GameAction::Move { to_room: 1 }); // Room doesn't matter
    let result = apply_action(state, "bogus_player", move_action);

    // 3. Assert: The action fails with a PlayerNotFound error.
    assert!(
        result.is_err(),
        "Action should fail for a player that does not exist."
    );
    let error = result.unwrap_err();
    assert!(
        matches!(error, GameError::PlayerNotFound),
        "Expected PlayerNotFound, got {:?}",
        error
    );
}

#[test]
fn test_move_to_nonexistent_room() {
    // 1. Setup: Create a game.
    let mut state = GameLogic::new_game(vec!["p1".to_owned()], 0);
    state.phase = GamePhase::TacticalPlanning;
    let player_id = state.players.keys().next().unwrap().clone();

    // 2. Action: Attempt to move to a room ID that is guaranteed not to exist.
    let move_action = Action::Game(GameAction::Move { to_room: 999 });
    let result = apply_action(state, &player_id, move_action);

    // 3. Assert: The action fails with an InvalidMove error.
    assert!(
        result.is_err(),
        "Move should fail if the target room does not exist."
    );
    let error = result.unwrap_err();
    assert!(
        matches!(error, GameError::InvalidMove),
        "Expected InvalidMove, got {:?}",
        error
    );
}

#[test]
fn test_targeted_action_with_invalid_range() {
    // 1. Setup: Create a game with two players in rooms far from each other.
    let mut state = GameLogic::new_game(vec!["p1".to_owned(), "p2".to_owned()], 0);
    state.phase = GamePhase::TacticalPlanning;
    let p1_id = "p1".to_owned();
    let p2_id = "p2".to_owned();

    // Manually move p2 to a room that is not adjacent to p1's starting room (Dormitory, ID 2)
    // The Bridge (ID 7) is not adjacent to the Dormitory.
    let bridge_id = 7;
    let p2 = state.players.get_mut(&p2_id).unwrap();
    p2.room_id = bridge_id;

    let p1 = state.players.get_mut(&p1_id).unwrap();
    p1.inventory.push(ItemType::Peppernut); // Give p1 an item to throw

    // 2. Action & Assert: Attempt FirstAid and Throw actions that should fail.
    let first_aid_action = Action::Game(GameAction::FirstAid {
        target_player: p2_id.clone(),
    });
    let throw_action = Action::Game(GameAction::Throw {
        target_player: p2_id.clone(),
        item_index: 0,
    });

    let aid_result = apply_action(state.clone(), &p1_id, first_aid_action);
    let throw_result = apply_action(state.clone(), &p1_id, throw_action);

    assert!(
        aid_result.is_err(),
        "FirstAid should fail if the target player is not in range."
    );
    assert!(
        throw_result.is_err(),
        "Throw should fail if the target player is not in range."
    );
}
