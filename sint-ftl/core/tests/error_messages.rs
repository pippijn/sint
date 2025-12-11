use sint_core::{
    logic::GameLogic,
    types::{Action, GameAction, GamePhase, ItemType},
    GameError,
};

#[test]
fn test_bake_wrong_room_msg() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Move P1 to Cannons (6 in Star Layout)
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 6;
        p.ap = 2;
    }

    let res = GameLogic::apply_action(state, "P1", Action::Game(GameAction::Bake), None);

    match res {
        Err(GameError::InvalidAction(msg)) => {
            assert!(
                msg.contains("Bake requires Kitchen (Room 5)"),
                "Msg: {}",
                msg
            );
            assert!(msg.contains("but you are in Cannons (6)"), "Msg: {}", msg);
        }
        _ => panic!("Expected InvalidAction error, got {:?}", res),
    }
}

#[test]
fn test_shoot_wrong_room_msg() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Move P1 to Bridge (7 in Star Layout)
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 7;
        p.ap = 2;
        p.inventory.push(ItemType::Peppernut); // Give ammo so checks pass until room check
    }

    let res = GameLogic::apply_action(state, "P1", Action::Game(GameAction::Shoot), None);

    match res {
        Err(GameError::InvalidAction(msg)) => {
            assert!(
                msg.contains("Shoot requires Cannons (Room 6)"),
                "Msg: {}",
                msg
            );
            assert!(msg.contains("but you are in Bridge (7)"), "Msg: {}", msg);
        }
        _ => panic!("Expected InvalidAction error, got {:?}", res),
    }
}
