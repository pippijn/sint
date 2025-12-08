use sint_core::{Action, GameError, GameLogic, GamePhase};

#[test]
fn test_move_in_lobby() {
    let state = GameLogic::new_game(vec!["Player1".to_string()], 12345);

    assert_eq!(state.phase, GamePhase::Lobby);
    let player = state.players.get("Player1").unwrap();
    assert_eq!(player.room_id, sint_core::types::SystemType::Dormitory.as_u32());
    assert_eq!(player.ap, 2);

    // Try to move to Hallway (7)
    let action = Action::Move {
        to_room: sint_core::types::SystemType::Hallway.as_u32(),
    };

    // This should now FAIL because we are in Lobby
    let res = GameLogic::apply_action(state.clone(), "Player1", action, None);

    match res {
        Ok(_) => {
            panic!("Action should have failed in Lobby!");
        }
        Err(GameError::InvalidAction(msg)) => {
            assert_eq!(msg, "Cannot act during Lobby");
        }
        Err(e) => {
            panic!("Wrong error type: {:?}", e);
        }
    }
}
