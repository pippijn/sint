use sint_core::{
    logic::{apply_action, pathfinding::find_path, GameLogic},
    types::{Action, GameAction, GamePhase, MapLayout, MetaAction, SystemType},
};

#[test]
fn test_set_layout_resets_readiness() {
    let mut state = GameLogic::new_game(vec!["P1".to_string(), "P2".to_string()], 12345);

    // 1. P1 votes Ready
    state = apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
    )
    .unwrap();

    assert!(state.players["P1"].is_ready);
    assert!(!state.players["P2"].is_ready);

    // 2. Change Layout
    state = apply_action(
        state,
        "P1",
        Action::Meta(MetaAction::SetMapLayout {
            layout: MapLayout::Torus,
        }),
    )
    .unwrap();

    // 3. Verify P1 is no longer ready
    assert!(
        !state.players["P1"].is_ready,
        "Changing map should reset ready status"
    );
    assert_eq!(state.layout, MapLayout::Torus);
}

#[test]
fn test_set_layout_moves_players_to_dormitory() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);

    // Move P1 to Room 9 (Storage in Star)
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 9;
    }

    // Change to Torus
    state = apply_action(
        state,
        "P1",
        Action::Meta(MetaAction::SetMapLayout {
            layout: MapLayout::Torus,
        }),
    )
    .unwrap();

    // Verify P1 is in the new Dormitory (Room 1 in Torus)
    let p1 = &state.players["P1"];
    let dorm_room = state.map.rooms.get(&p1.room_id).unwrap();

    assert_eq!(dorm_room.system, Some(SystemType::Dormitory));
    assert_ne!(p1.room_id, 9, "Player should have moved from Room 9");
}

#[test]
fn test_torus_wrap_around_movement() {
    // Initialize directly with Torus
    let state = GameLogic::new_game_with_layout(vec!["P1".to_string()], 12345, MapLayout::Torus);

    // Torus has 12 rooms (0-11). 0 and 11 should be neighbors.
    let room_0 = 0;
    let room_11 = 11;

    let path = find_path(&state.map, room_0, room_11).expect("Path should exist");

    // Path should be 1 step (Direct neighbor)
    assert_eq!(
        path.len(),
        1,
        "Should take the short way around the ring (0 -> 11)"
    );
    assert_eq!(path[0], 11);
}

#[test]
fn test_cannot_change_layout_mid_game() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);

    // Advance to MorningReport
    state.phase = GamePhase::MorningReport;

    let res = apply_action(
        state,
        "P1",
        Action::Meta(MetaAction::SetMapLayout {
            layout: MapLayout::Torus,
        }),
    );

    assert!(res.is_err(), "Should not allow layout change outside Lobby");
}

#[test]
fn test_full_sync_preserves_layout() {
    let state_torus =
        GameLogic::new_game_with_layout(vec!["P1".to_string()], 12345, MapLayout::Torus);

    let json = serde_json::to_string(&state_torus).unwrap();

    // Start with default (Star)
    let state_empty = GameLogic::new_game(vec![], 0);
    assert_eq!(state_empty.layout, MapLayout::Star);

    // Sync
    let state_synced = apply_action(
        state_empty,
        "P1",
        Action::Meta(MetaAction::FullSync { state_json: json }),
    )
    .unwrap();

    assert_eq!(state_synced.layout, MapLayout::Torus);
    assert_eq!(state_synced.map.rooms.len(), 12);
}
