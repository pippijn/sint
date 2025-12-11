use sint_core::{
    types::{Action, GameAction},
    GameLogic, GamePhase,
};

#[test]
fn test_deterministic_action_ids() {
    let state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    // Move to TacticalPlanning
    let mut state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap(); // Lobby -> Morning
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap(); // Morning -> Telegraph
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap(); // Telegraph -> Planning

    assert_eq!(state.phase, GamePhase::TacticalPlanning);

    // Fork State
    let state1 = state.clone();
    let state2 = state.clone();

    // Action 1: Move to Hallway
    let action = Action::Game(GameAction::Move { to_room: 0 });

    // Apply on State 1
    let state1_prime = GameLogic::apply_action(state1, "P1", action.clone(), None).unwrap();
    let id1 = state1_prime.proposal_queue.last().unwrap().id.clone();

    // Apply on State 2
    let state2_prime = GameLogic::apply_action(state2, "P1", action.clone(), None).unwrap();
    let id2 = state2_prime.proposal_queue.last().unwrap().id.clone();

    // Assert IDs match
    assert_eq!(id1, id2, "Action IDs must be deterministic");

    // Assert Seeds match
    assert_eq!(
        state1_prime.rng_seed, state2_prime.rng_seed,
        "RNG state must remain synced"
    );

    // Action 2: Subsequent Move
    let kitchen_id = sint_core::logic::find_room_with_system_in_map(
        &state1_prime.map,
        sint_core::types::SystemType::Kitchen,
    )
    .unwrap();
    let action3 = Action::Game(GameAction::Move {
        to_room: kitchen_id,
    });

    let state1_double = GameLogic::apply_action(state1_prime, "P1", action3.clone(), None).unwrap();
    let id1_next = state1_double.proposal_queue.last().unwrap().id.clone();

    let state2_double = GameLogic::apply_action(state2_prime, "P1", action3.clone(), None).unwrap();
    let id2_next = state2_double.proposal_queue.last().unwrap().id.clone();

    assert_eq!(
        id1_next, id2_next,
        "Subsequent Action IDs must be deterministic"
    );
    assert_ne!(id1, id1_next, "Different actions should have different IDs");
}
