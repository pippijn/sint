use sint_core::{
    GameLogic,
    logic::{find_room_with_system_in_map, resolution},
    types::*,
};

#[test]
fn test_system_breaks_faster_with_multiple_fires() {
    let player_ids = vec!["P1".to_owned()];
    let seed = 42;
    let mut state = GameLogic::new_game(player_ids, seed);
    state.phase = GamePhase::EnemyAction;

    let kitchen_id = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();
    let bridge_id = find_room_with_system_in_map(&state.map, SystemType::Bridge).unwrap();

    // Setup: Kitchen has 1 Fire, Bridge has 2 Fires.
    // Kitchen should lose 1 health per round.
    // Bridge should lose 2 health per round.
    if let Some(room) = state.map.rooms.get_mut(&kitchen_id) {
        room.hazards.push(HazardType::Fire);
        assert_eq!(room.system_health, SYSTEM_HEALTH);
    }
    if let Some(room) = state.map.rooms.get_mut(&bridge_id) {
        room.hazards.push(HazardType::Fire);
        room.hazards.push(HazardType::Fire);
        assert_eq!(room.system_health, SYSTEM_HEALTH);
    }

    // Resolve Hazards once
    resolution::resolve_hazards(&mut state);

    assert_eq!(
        state.map.rooms[&kitchen_id].system_health,
        SYSTEM_HEALTH - 1,
        "Kitchen (1 fire) should have lost 1 health"
    );
    assert_eq!(
        state.map.rooms[&bridge_id].system_health,
        SYSTEM_HEALTH - 2,
        "Bridge (2 fires) should have lost 2 health"
    );
    assert!(
        !state.map.rooms[&bridge_id].is_broken,
        "Bridge should not be broken yet"
    );

    // Resolve Hazards again
    resolution::resolve_hazards(&mut state);

    assert_eq!(
        state.map.rooms[&kitchen_id].system_health,
        SYSTEM_HEALTH - 2,
        "Kitchen (1 fire) should have lost 2 health total"
    );
    // Bridge health was 1. 1 - 2 = 0 (and is_broken = true)
    assert_eq!(
        state.map.rooms[&bridge_id].system_health, 0,
        "Bridge (2 fires) should have 0 health"
    );
    assert!(
        state.map.rooms[&bridge_id].is_broken,
        "Bridge should be broken after receiving more damage than remaining health"
    );
}
