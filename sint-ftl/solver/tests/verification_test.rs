use sint_core::types::{GameAction, ItemType};
use sint_solver::verification::parse_game_action;

#[test]
fn test_parse_game_action() {
    assert_eq!(parse_game_action("Move 5"), GameAction::Move { to_room: 5 });
    assert_eq!(parse_game_action("Bake"), GameAction::Bake);
    assert_eq!(parse_game_action("Shoot"), GameAction::Shoot);
    assert_eq!(
        parse_game_action("Throw P1 0"),
        GameAction::Throw {
            target_player: "P1".to_string(),
            item_index: 0
        }
    );
    assert_eq!(parse_game_action("Extinguish"), GameAction::Extinguish);
    assert_eq!(parse_game_action("Repair"), GameAction::Repair);
    assert_eq!(
        parse_game_action("PickUp Peppernut"),
        GameAction::PickUp {
            item_type: ItemType::Peppernut
        }
    );
    assert_eq!(
        parse_game_action("PickUp Extinguisher"),
        GameAction::PickUp {
            item_type: ItemType::Extinguisher
        }
    );
    assert_eq!(
        parse_game_action("Drop 1"),
        GameAction::Drop { item_index: 1 }
    );
    assert_eq!(parse_game_action("Pass"), GameAction::Pass);
    assert_eq!(
        parse_game_action("Ready"),
        GameAction::VoteReady { ready: true }
    );
    assert_eq!(
        parse_game_action("VoteReady"),
        GameAction::VoteReady { ready: true }
    );
    assert_eq!(parse_game_action("RaiseShields"), GameAction::RaiseShields);
    assert_eq!(
        parse_game_action("EvasiveManeuvers"),
        GameAction::EvasiveManeuvers
    );
    assert_eq!(parse_game_action("Lookout"), GameAction::Lookout);
    assert_eq!(parse_game_action("Interact"), GameAction::Interact);
    assert_eq!(
        parse_game_action("Revive P2"),
        GameAction::Revive {
            target_player: "P2".to_string()
        }
    );
    assert_eq!(
        parse_game_action("FirstAid P3"),
        GameAction::FirstAid {
            target_player: "P3".to_string()
        }
    );
    assert_eq!(
        parse_game_action("Chat Hello World"),
        GameAction::Chat {
            message: "Hello World".to_string()
        }
    );
}

#[test]
fn test_run_verification_smoke() {
    use sint_core::logic::GameLogic;
    use sint_solver::verification::run_verification;

    let player_ids = vec!["P1".to_string(), "P2".to_string()];
    let state = GameLogic::new_game(player_ids, 42);

    // Create a simple solution: both players pass in Round 1 and Round 2
    let solution = vec![
        vec![
            ("P1".to_string(), GameAction::Pass),
            ("P2".to_string(), GameAction::Pass),
        ],
        vec![
            ("P1".to_string(), GameAction::Pass),
            ("P2".to_string(), GameAction::Pass),
        ],
    ];

    let result = run_verification(state, solution);

    // It should not be a victory yet, but it should be successful in terms of execution
    assert!(!result.success);
    assert!(result.error.is_none());
    // After two rounds of actions, it should have reached at least Round 2's TacticalPlanning
    // or even finished it and be in Execution of Round 2.
    assert!(result.final_state.turn_count >= 2);
}

#[test]
fn test_run_verification_unfinished_round() {
    use sint_core::logic::GameLogic;
    use sint_solver::verification::run_verification;

    let player_ids = vec!["P1".to_string()];
    let state = GameLogic::new_game(player_ids, 42);

    // P1 has 2 AP. This block only provides 1 Move action (1 AP).
    // The round is not finished.
    let solution = vec![vec![("P1".to_string(), GameAction::Move { to_room: 1 })]];

    let result = run_verification(state, solution);

    assert!(!result.success);
    assert!(result.error.is_some());
    if let Some(sint_core::GameError::InvalidAction(msg)) = &result.error {
        assert!(msg.contains("but players still have AP"));
    } else {
        panic!(
            "Expected InvalidAction error with AP message, got {:?}",
            result.error
        );
    }
}
