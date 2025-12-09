use sint_core::{Action, CardId, GameLogic, GamePhase, HazardType, ItemType};

// S02: Fire Drill
// Goal: Extinguish fires.
#[test]
fn test_scenario_fire_drill() {
    let mut state = GameLogic::new_game(vec!["P1".to_string(), "P2".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Setup Scenario
    // P1 in Room 3 (Dormitory), P2 in Room 7 (Hallway)
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 3;
        p.ap = 2;
    }
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = 7;
        p.ap = 2;
    }

    // Hazards: 2 Fire in Kitchen (6), 1 Fire in Cargo (4)
    if let Some(r) = state.map.rooms.get_mut(&6) {
        r.hazards = vec![HazardType::Fire, HazardType::Fire];
    }
    if let Some(r) = state.map.rooms.get_mut(&4) {
        r.hazards = vec![HazardType::Fire];
    }

    // Turn 1 Execution
    // P2: Move 7->6. Extinguish.
    state = GameLogic::apply_action(state, "P2", Action::Move { to_room: 6 }, None).unwrap();
    state = GameLogic::apply_action(state, "P2", Action::Extinguish, None).unwrap();

    // P1: Move 3->7->4.
    state = GameLogic::apply_action(state, "P1", Action::Move { to_room: 7 }, None).unwrap();
    state = GameLogic::apply_action(state, "P1", Action::Move { to_room: 4 }, None).unwrap();

    // Resolve Batch
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    state = GameLogic::apply_action(state, "P2", Action::VoteReady { ready: true }, None).unwrap();

    // Advance to next Planning Phase
    // Current: Execution. Next: EnemyAction -> Morning -> Telegraph -> Planning.
    // Note: We need to handle VoteReady repeatedly.

    // We expect P1 and P2 to be alive (HP was 3, took 1 dmg from fire in EnemyAction).
    // P2 HP: 2. P1 HP: 2.

    fn advance_to_planning(mut state: sint_core::GameState) -> sint_core::GameState {
        let mut attempts = 0;
        while state.phase != GamePhase::TacticalPlanning && state.phase != GamePhase::GameOver {
            let pids: Vec<String> = state.players.keys().cloned().collect();
            for pid in pids {
                // Ignore errors (e.g. if already ready)
                if let Ok(new_state) = GameLogic::apply_action(
                    state.clone(),
                    &pid,
                    Action::VoteReady { ready: true },
                    None,
                ) {
                    state = new_state;
                }
            }
            attempts += 1;
            if attempts > 20 {
                panic!("Stuck in loop phase: {:?}", state.phase);
            }
        }
        state
    }

    state = advance_to_planning(state);

    assert_ne!(state.phase, GamePhase::GameOver);
    assert_eq!(state.players["P1"].hp, 2);
    assert_eq!(state.players["P2"].hp, 2);

    // Turn 2: Extinguish remaining
    state = GameLogic::apply_action(state, "P2", Action::Extinguish, None).unwrap(); // Kitchen clear
    state = GameLogic::apply_action(state, "P1", Action::Extinguish, None).unwrap(); // Cargo clear

    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    state = GameLogic::apply_action(state, "P2", Action::VoteReady { ready: true }, None).unwrap();

    // Check hazards
    // Execution phase resolves immediately.
    assert!(state.map.rooms[&6].hazards.is_empty());
    assert!(state.map.rooms[&4].hazards.is_empty());
}

// S03: Bucket Brigade
// Goal: Transfer item across chain in one turn.
#[test]
fn test_scenario_bucket_brigade() {
    let mut state = GameLogic::new_game(
        vec!["P1".to_string(), "P2".to_string(), "P3".to_string()],
        12345,
    );
    state.phase = GamePhase::TacticalPlanning;

    // Setup
    // P1 in Kitchen(6), has Nut.
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 6;
        p.inventory = vec![ItemType::Peppernut];
        p.ap = 2;
    }
    // P2 in Hallway(7).
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = 7;
        p.inventory = vec![];
        p.ap = 2;
    }
    // P3 in Cannons(8).
    if let Some(p) = state.players.get_mut("P3") {
        p.room_id = 8;
        p.inventory = vec![];
        p.ap = 2;
    }

    // P1 Throws to P2
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Throw {
            target_player: "P2".to_string(),
            item_index: 0,
        },
        None,
    )
    .unwrap();

    // P2 Throws to P3 (Dependent on P1's action)
    state = GameLogic::apply_action(
        state,
        "P2",
        Action::Throw {
            target_player: "P3".to_string(),
            item_index: 0,
        },
        None,
    )
    .unwrap();

    // Resolve
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    state = GameLogic::apply_action(state, "P2", Action::VoteReady { ready: true }, None).unwrap();
    state = GameLogic::apply_action(state, "P3", Action::VoteReady { ready: true }, None).unwrap();

    // Check P3 has nut
    assert_eq!(state.players["P3"].inventory.len(), 1);
    assert_eq!(state.players["P3"].inventory[0], ItemType::Peppernut);

    // Check P1/P2 empty
    assert!(state.players["P1"].inventory.is_empty());
    assert!(state.players["P2"].inventory.is_empty());
}

// S04: The Kraken (Boss Fight)
#[test]
fn test_scenario_kraken_fight() {
    let mut state = GameLogic::new_game(
        vec![
            "P1".to_string(),
            "P2".to_string(),
            "P3".to_string(),
            "P4".to_string(),
        ],
        12345,
    );
    state.phase = GamePhase::TacticalPlanning;

    // Setup
    state.hull_integrity = 10; // 50% of 20
    state.boss_level = 3;
    state.enemy = sint_core::logic::get_boss(3);
    assert_eq!(state.enemy.name, "The Kraken");

    // Fog Bank Active
    let fog = state
        .deck
        .iter()
        .find(|c| c.id == CardId::FogBank)
        .cloned()
        .unwrap();
    state.active_situations.push(fog);

    // Leaking Engine (Room 5)
    if let Some(r) = state.map.rooms.get_mut(&5) {
        r.hazards.push(HazardType::Water);
    }

    // Verify State
    assert_eq!(state.active_situations.len(), 1);
    assert_eq!(state.active_situations[0].id, CardId::FogBank);

    // P1 Shoot Kraken.
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 8;
        p.inventory = vec![ItemType::Peppernut];
        p.ap = 2;
    }

    state = GameLogic::apply_action(state, "P1", Action::Shoot, None).unwrap();

    // Resolve
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    state = GameLogic::apply_action(state, "P2", Action::VoteReady { ready: true }, None).unwrap();
    state = GameLogic::apply_action(state, "P3", Action::VoteReady { ready: true }, None).unwrap();
    GameLogic::apply_action(state, "P4", Action::VoteReady { ready: true }, None).unwrap();
}
