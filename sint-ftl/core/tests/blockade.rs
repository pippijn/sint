use sint_core::logic::{GameLogic, actions::get_valid_actions, cards::get_behavior};
use sint_core::types::{Action, CardId, GameAction, GamePhase, MapLayout, SystemType};

#[test]
fn test_blockade_prevents_move_to_cannons_in_valid_actions() {
    // 1. Setup Game
    let mut state = GameLogic::new_game_with_layout(
        vec!["P1".to_string(), "P2".to_string()],
        12345,
        MapLayout::Star,
    );

    // 2. Advance to TacticalPlanning
    state.phase = GamePhase::TacticalPlanning;

    // 3. Inject Blockade Card
    let blockade = get_behavior(CardId::Blockade).get_struct();
    state.active_situations.push(blockade);

    // 4. Position P1 adjacent to Cannons (Room 6 in Star map usually, or we search)
    let cannons_id = state
        .map
        .rooms
        .values()
        .find(|r| r.system == Some(SystemType::Cannons))
        .map(|r| r.id)
        .expect("Cannons not found");

    let cannons_room = state.map.rooms.get(&cannons_id).unwrap();
    let neighbor_id = cannons_room.neighbors[0]; // Pick a neighbor

    state.players.get_mut("P1").unwrap().room_id = neighbor_id;
    state.players.get_mut("P1").unwrap().ap = 2;

    // 5. Get Valid Actions
    let actions = get_valid_actions(&state, "P1");

    // 6. Assert that "Move to Cannons" is NOT in the list
    let move_to_cannons = Action::Game(GameAction::Move {
        to_room: cannons_id,
    });

    let contains_illegal_move = actions.contains(&move_to_cannons);
    assert!(
        !contains_illegal_move,
        "get_valid_actions should NOT return a move to Cannons when Blockade is active"
    );

    // 7. Assert that "Pass" IS in the list
    let pass_action = Action::Game(GameAction::Pass);
    assert!(
        actions.contains(&pass_action),
        "get_valid_actions SHOULD return Pass"
    );
}
