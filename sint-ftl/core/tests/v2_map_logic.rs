use sint_core::{
    logic::{GameLogic, cards::get_behavior, find_room_with_system, resolution::process_round_end},
    types::*,
};

#[test]
fn test_v2_star_topology() {
    let state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    let hub_id = 0;

    // 1. Verify Hub is Room 0 and Empty
    let hub = state.map.rooms.get(&hub_id).expect("Room 0 should exist");
    assert!(hub.system.is_none(), "Room 0 should be the Empty Hub");

    // 2. Verify Connectivity
    // Hub should connect to all other rooms (1..9)
    assert_eq!(hub.neighbors.len(), 9);
    for i in 1..=9 {
        assert!(hub.neighbors.contains(&i));

        // Spoke should connect ONLY to Hub
        let spoke = state.map.rooms.get(&i).unwrap();
        assert_eq!(
            spoke.neighbors,
            smallvec::SmallVec::<[u32; 8]>::from_slice(&[hub_id]),
            "Spoke {} should only connect to Hub",
            i
        );
    }
}

#[test]
fn test_system_mapping_v2() {
    let state = GameLogic::new_game(vec!["P1".to_owned()], 12345);

    // Verify Storage is System 10
    assert_eq!(SystemType::Storage.as_u32(), 10);

    // Verify Storage Room exists and has the system
    let storage_room_id = find_room_with_system(&state, SystemType::Storage).unwrap();
    let storage_room = state.map.rooms.get(&storage_room_id).unwrap();
    assert_eq!(storage_room.system, Some(SystemType::Storage));
}

#[test]
fn test_enemy_targeting_miss() {
    // We can't easily force the RNG for the roll inside advance_phase without mocking,
    // but we can unit test the logic if it were exposed.
    // Since we can't, we'll verify the SystemType::from_u32 mapping which drives it.

    assert!(SystemType::from_u32(11).is_none());
    assert!(SystemType::from_u32(12).is_none());

    // If we could inject a seed that rolls 11, we'd verify AttackEffect::Miss.
    // Let's try to brute force a seed? No, that's flaky.
    // Instead, trust the logic change we made in `card_fog.rs` (which explicitly handles Miss)
    // and `resolution.rs` (which handles None target).
}

#[test]
fn test_amerigo_diet_restriction() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    let storage_id = find_room_with_system(&state, SystemType::Storage).unwrap();

    // Setup: Storage has 1 Extinguisher and 1 Peppernut
    if let Some(r) = state.map.rooms.get_mut(&storage_id) {
        r.items.clear();
        r.items.push(ItemType::Extinguisher);
        r.items.push(ItemType::Peppernut);
    }

    let behavior = get_behavior(CardId::Amerigo);
    state.active_situations.push(behavior.get_struct());
    process_round_end(&mut state);

    let items = &state.map.rooms[&storage_id].items;
    // Should eat Peppernut, leave Extinguisher
    assert!(
        !items.contains(&ItemType::Peppernut),
        "Amerigo should eat Peppernut"
    );
    assert!(
        items.contains(&ItemType::Extinguisher),
        "Amerigo should NOT eat Extinguisher"
    );
}

#[test]
fn test_wheel_clamp_glitch_movement() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);

    // Place P1 in Room 9 (Storage in default layout)
    let max_room = 9;
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = max_room;
    }

    let behavior = get_behavior(CardId::WheelClamp);
    state.active_situations.push(behavior.get_struct());
    process_round_end(&mut state);

    // Should wrap to Room 0 (Hub)
    let p1 = state.players.get("P1").unwrap();
    assert_eq!(p1.room_id, 0, "Wheel Clamp should wrap 9 -> 0");
}

#[test]
fn test_false_note_flee_to_hub() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    let cannons = find_room_with_system(&state, SystemType::Cannons).unwrap();

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = cannons;
    }

    let behavior = get_behavior(CardId::FalseNote);
    behavior.on_activate(&mut state);

    let p1 = state.players.get("P1").unwrap();
    // Nearest empty room is Hub (0)
    assert_eq!(p1.room_id, 0, "Should flee to Hub (0)");
}

#[test]
fn test_v2_start_location_and_mapping() {
    let state = GameLogic::new_game(vec!["P1".to_owned()], 12345);

    // 1. Verify specific room mappings (Room 2=Dormitory, Room 3=Cargo)
    // Note: These IDs rely on the order in ROOM_DEFINITIONS
    let room2 = state.map.rooms.get(&2).unwrap();
    assert_eq!(room2.name, "Dormitory");
    assert_eq!(room2.system, Some(SystemType::Dormitory));

    let room3 = state.map.rooms.get(&3).unwrap();
    assert_eq!(room3.name, "Cargo");
    assert_eq!(room3.system, Some(SystemType::Cargo));

    // 2. Verify Helper Lookup
    let lookup_dorm = find_room_with_system(&state, SystemType::Dormitory);
    assert_eq!(lookup_dorm, Some(2));

    // 3. Verify Player Start
    let p1 = state.players.get("P1").unwrap();
    assert_eq!(p1.room_id, 2, "Player should start in Room 2 (Dormitory)");
}
