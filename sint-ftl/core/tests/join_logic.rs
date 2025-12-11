use sint_core::logic::{apply_action, GameLogic};
use sint_core::types::{Action, MetaAction, SystemType};

#[test]
fn test_join_action_spawns_in_dormitory() {
    // 1. Setup: Create a new game with no players
    let initial_state = GameLogic::new_game(vec![], 0);
    
    // Find the canonical Dormitory room ID from the map itself
    let dormitory_id = initial_state
        .map
        .rooms
        .values()
        .find(|r| r.system == Some(SystemType::Dormitory))
        .expect("Game map should have a Dormitory")
        .id;

    // 2. Action: A new player joins the game
    let player_id = "player_joining".to_string();
    let join_action = Action::Meta(MetaAction::Join {
        name: "Joiner".to_string(),
    });

    let final_state = apply_action(initial_state, &player_id, join_action).unwrap();

    // 3. Assert: The new player is in the Dormitory
    let player = final_state.players.get(&player_id).unwrap();
    assert_eq!(
        player.room_id, dormitory_id,
        "Player should spawn in the Dormitory (Room {}) but was in {}",
        dormitory_id, player.room_id
    );
}
