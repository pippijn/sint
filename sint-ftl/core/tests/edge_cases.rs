use sint_core::{GameLogic, logic::find_room_with_system_in_map, types::*};

#[test]
fn test_first_aid_cannot_heal_fainted() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned(), "P2".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let sickbay = find_room_with_system_in_map(&state.map, SystemType::Sickbay).unwrap();

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sickbay;
        p.ap = 2;
    }
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = sickbay;
        p.hp = 0;
        p.status.push(PlayerStatus::Fainted);
    }

    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::FirstAid {
            target_player: "P2".to_owned(),
        }),
        None,
    );

    // It should probably be an error or at least not remove Fainted status.
    // If it succeeds but P2 is still Fainted, it's also arguably correct but confusing.
    // Most FTL-like games require a specific "Revive" or "Repair" for downed units.
    assert!(
        res.is_err(),
        "First Aid should not work on Fainted players. Use Revive instead."
    );
}

#[test]
fn test_fire_damage_during_rest_round() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned()], 12345);
    state.is_resting = true;
    state.phase = GamePhase::TacticalPlanning;
    state.hull_integrity = 20;

    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();
    if let Some(r) = state.map.rooms.get_mut(&kitchen) {
        r.hazards.push(HazardType::Fire);
    }

    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 0; // Force advance
    }

    // P -> E
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();
    // E -> EA (Resolves hazards)
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    assert_eq!(
        state.hull_integrity, 19,
        "Hull should still take damage from fire during rest rounds"
    );
}

#[test]
fn test_revive_restores_1_hp_only() {
    let mut state = GameLogic::new_game(vec!["P1".to_owned(), "P2".to_owned()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    let dormitory = find_room_with_system_in_map(&state.map, SystemType::Dormitory).unwrap();

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = dormitory;
        p.ap = 2;
    }
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = dormitory;
        p.hp = 0;
        p.status.push(PlayerStatus::Fainted);
    }

    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Revive {
            target_player: "P2".to_owned(),
        }),
        None,
    );
    assert!(res.is_ok());
    let mut state = res.unwrap();

    // Resolve
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();
    state = GameLogic::apply_action(
        state,
        "P2",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    let p2 = &state.players["P2"];
    assert_eq!(p2.hp, 1, "Revive should only restore 1 HP");
    assert!(!p2.status.contains(&PlayerStatus::Fainted));
}
