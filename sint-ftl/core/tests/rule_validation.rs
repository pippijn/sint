use sint_core::{
    GameLogic,
    logic::{find_room_with_system_in_map, resolution},
    types::*,
};

#[test]
fn test_repair_priority_hierarchy() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let cargo_id = find_room_with_system_in_map(&state.map, SystemType::Cargo).unwrap();

    // Setup: Cargo room has Water, Damaged System (1/3 HP), and Ship Hull is damaged.
    if let Some(r) = state.map.rooms.get_mut(&cargo_id) {
        r.hazards.push(HazardType::Water);
        r.system_health = 1;
        r.is_broken = true;
    }
    state.hull_integrity = 10;

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = cargo_id;
        p.ap = 10;
    }

    // --- REPAIR 1: Should remove WATER ---
    state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::Repair), None).unwrap();
    resolution::resolve_proposal_queue(&mut state, false);

    assert!(
        !state.map.rooms[&cargo_id]
            .hazards
            .contains(&HazardType::Water),
        "First repair should remove water"
    );
    assert_eq!(
        state.map.rooms[&cargo_id].system_health, 1,
        "System health should not have changed yet"
    );
    assert_eq!(state.hull_integrity, 10, "Hull should not have changed yet");

    // --- REPAIR 2: Should restore SYSTEM HP ---
    state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::Repair), None).unwrap();
    resolution::resolve_proposal_queue(&mut state, false);

    assert_eq!(
        state.map.rooms[&cargo_id].system_health, 2,
        "Second repair should restore system HP"
    );
    assert_eq!(
        state.hull_integrity, 10,
        "Hull should still not have changed"
    );

    // --- REPAIR 3: Should restore SYSTEM HP to full ---
    state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::Repair), None).unwrap();
    resolution::resolve_proposal_queue(&mut state, false);

    assert_eq!(
        state.map.rooms[&cargo_id].system_health, 3,
        "Third repair should restore system HP to full"
    );
    assert!(
        !state.map.rooms[&cargo_id].is_broken,
        "System should no longer be broken"
    );
    assert_eq!(
        state.hull_integrity, 10,
        "Hull should still not have changed"
    );

    // --- REPAIR 4: Should restore HULL ---
    state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::Repair), None).unwrap();
    resolution::resolve_proposal_queue(&mut state, false);

    assert_eq!(
        state.hull_integrity, 11,
        "Fourth repair should finally restore Hull integrity in Cargo"
    );
}

#[test]
fn test_system_roll_to_room_mapping() {
    // Star layout mapping check:
    // Roll 2 -> Bow -> Room 1
    // Roll 3 -> Dormitory -> Room 2
    // Roll 10 -> Storage -> Room 9

    let state = GameLogic::new_game_with_layout(vec!["P1".to_owned()], 12345, MapLayout::Star);

    // Helper to simulate a specific roll for telegraphing
    let simulate_roll = |roll: u32| {
        let mut state = state.clone();
        state.phase = GamePhase::MorningReport;

        // We bypass the RNG by manually setting the next telegraph
        if let Some(sys) = SystemType::from_u32(roll) {
            let room_id = find_room_with_system_in_map(&state.map, sys);
            state.enemy.next_attack = Some(EnemyAttack {
                target_room: room_id,
                target_system: Some(sys),
                effect: AttackEffect::Fireball,
            });
        }
        state.enemy.next_attack.unwrap()
    };

    let attack_roll_2 = simulate_roll(2);
    assert_eq!(attack_roll_2.target_system, Some(SystemType::Bow));
    assert_eq!(
        attack_roll_2.target_room,
        Some(1),
        "Roll 2 (Bow) should target Room 1 in Star layout"
    );

    let attack_roll_3 = simulate_roll(3);
    assert_eq!(attack_roll_3.target_system, Some(SystemType::Dormitory));
    assert_eq!(
        attack_roll_3.target_room,
        Some(2),
        "Roll 3 (Dormitory) should target Room 2 in Star layout"
    );

    let attack_roll_10 = simulate_roll(10);
    assert_eq!(attack_roll_10.target_system, Some(SystemType::Storage));
    assert_eq!(
        attack_roll_10.target_room,
        Some(9),
        "Roll 10 (Storage) should target Room 9 in Star layout"
    );
}
