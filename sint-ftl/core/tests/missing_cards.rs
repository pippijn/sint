use sint_core::{
    GameError,
    logic::{GameLogic, cards::get_behavior, find_room_with_system},
    types::*,
};

fn create_test_state() -> GameState {
    let players = vec!["P1".to_owned(), "P2".to_owned()];
    GameLogic::new_game(players, 12345)
}

#[test]
fn test_attack_wave() {
    let state = create_test_state();
    let behavior = get_behavior(CardId::AttackWave);
    assert_eq!(behavior.get_enemy_attack_count(&state), 2);
}

#[test]
fn test_big_leak() {
    let mut state = create_test_state();
    let behavior = get_behavior(CardId::BigLeak);
    let cargo_id = find_room_with_system(&state, SystemType::Cargo).unwrap();

    behavior.on_round_start(&mut state);

    let room = state.map.rooms.get(&cargo_id).unwrap();
    assert!(room.hazards.contains(&HazardType::Water));
}

#[test]
fn test_clogged_pipe() {
    let state = create_test_state();
    let behavior = get_behavior(CardId::CloggedPipe);

    let res = behavior.validate_action(&state, "P1", &GameAction::Bake);
    assert!(res.is_err());
}

#[test]
fn test_stowaway_trigger() {
    let mut state = create_test_state();
    let behavior = get_behavior(CardId::Stowaway);

    if let Some(p) = state.players.get_mut("P1") {
        p.inventory.push(ItemType::Peppernut);
    }

    behavior.on_trigger(&mut state);

    let p1 = state.players.get("P1").unwrap();
    assert!(p1.inventory.is_empty());
}

#[test]
fn test_falling_gift() {
    let mut state = create_test_state();
    let behavior = get_behavior(CardId::FallingGift);
    let cargo_id = find_room_with_system(&state, SystemType::Cargo).unwrap();

    behavior.on_activate(&mut state);

    let room = state.map.rooms.get(&cargo_id).unwrap();
    assert!(room.hazards.contains(&HazardType::Water));
    assert!(room.items.contains(&ItemType::Peppernut));
}

#[test]
fn test_high_pressure() {
    let mut state = create_test_state();
    let behavior = get_behavior(CardId::HighPressure);
    let initial_room = state.players["P1"].room_id;

    behavior.on_activate(&mut state);

    assert_ne!(state.players["P1"].room_id, initial_room);
}

#[test]
fn test_high_waves() {
    let mut state = create_test_state();
    let behavior = get_behavior(CardId::HighWaves);

    // Star layout: Dormitory (2) -> Hub (0) -> Engine (4)
    // Place P1 in Dormitory
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 2;
    }

    behavior.on_activate(&mut state);

    // Path to Engine (4) from Dormitory (2) is [0, 4]. First step is 0.
    assert_eq!(state.players["P1"].room_id, 0);
}

#[test]
fn test_jammed_cannon() {
    let state = create_test_state();
    let behavior = get_behavior(CardId::JammedCannon);

    let res = behavior.validate_action(&state, "P1", &GameAction::Shoot);
    assert!(res.is_err());
}

#[test]
fn test_leak() {
    let mut state = create_test_state();
    let behavior = get_behavior(CardId::Leak);
    let cargo_id = find_room_with_system(&state, SystemType::Cargo).unwrap();

    behavior.on_activate(&mut state);

    let room = state.map.rooms.get(&cargo_id).unwrap();
    assert!(room.hazards.contains(&HazardType::Water));
}

#[test]
fn test_lights_out() {
    let state = create_test_state();
    let behavior = get_behavior(CardId::LightsOut);

    let cost = behavior.modify_action_cost(&state, "P1", &GameAction::Move { to_room: 0 }, 1);
    assert_eq!(cost, 2);
}

#[test]
fn test_no_light() {
    let state = create_test_state();
    let behavior = get_behavior(CardId::NoLight);

    let res = behavior.validate_action(&state, "P1", &GameAction::Shoot);
    assert!(res.is_err());
}

#[test]
fn test_peppernut_rain() {
    let mut state = create_test_state();
    let behavior = get_behavior(CardId::PeppernutRain);
    let room_id = state.players["P1"].room_id;

    behavior.on_activate(&mut state);

    let room = state.map.rooms.get(&room_id).unwrap();
    assert!(room.items.contains(&ItemType::Peppernut));
}

#[test]
fn test_present() {
    let mut state = create_test_state();
    let behavior = get_behavior(CardId::Present);

    // Setup: 1 Fire
    if let Some(r) = state.map.rooms.get_mut(&0) {
        r.hazards.push(HazardType::Fire);
    }

    behavior.on_activate(&mut state);

    assert!(state.map.rooms[&0].hazards.is_empty());
}

#[test]
fn test_sing_a_song() {
    let mut state = create_test_state();
    let behavior = get_behavior(CardId::SingASong);

    // Setup: 1 Water
    if let Some(r) = state.map.rooms.get_mut(&0) {
        r.hazards.push(HazardType::Water);
    }

    behavior.on_activate(&mut state);

    assert!(state.map.rooms[&0].hazards.is_empty());
}

#[test]
fn test_static_noise() {
    let state = create_test_state();
    let behavior = get_behavior(CardId::StaticNoise);

    let res = behavior.validate_action(
        &state,
        "P1",
        &GameAction::Chat {
            message: "Hello".to_owned(),
        },
    );
    assert!(matches!(res, Err(GameError::Silenced)));

    let res_ok = behavior.validate_action(
        &state,
        "P1",
        &GameAction::Chat {
            message: "😊".to_owned(),
        },
    );
    assert!(res_ok.is_ok());
}

#[test]
fn test_strong_headwind() {
    let state = create_test_state();
    let behavior = get_behavior(CardId::StrongHeadwind);

    assert_eq!(behavior.get_hit_threshold(&state), 5);
}

#[test]
fn test_turbo_mode() {
    let mut state = create_test_state();
    let behavior = get_behavior(CardId::TurboMode);

    let ap_before = state.players["P1"].ap;
    behavior.on_round_start(&mut state);
    assert_eq!(state.players["P1"].ap, ap_before + 1);

    behavior.on_trigger(&mut state);
    let engine_id = find_room_with_system(&state, SystemType::Engine).unwrap();
    let room = state.map.rooms.get(&engine_id).unwrap();
    assert_eq!(
        room.hazards
            .iter()
            .filter(|&&h| h == HazardType::Fire)
            .count(),
        2
    );
}

#[test]
fn test_weird_gifts() {
    let mut state = create_test_state();
    let behavior = get_behavior(CardId::WeirdGifts);

    behavior.on_trigger(&mut state);

    let cargo_id = find_room_with_system(&state, SystemType::Cargo).unwrap();
    let sickbay_id = find_room_with_system(&state, SystemType::Sickbay).unwrap();

    assert_eq!(
        state.map.rooms[&cargo_id]
            .hazards
            .iter()
            .filter(|&&h| h == HazardType::Fire)
            .count(),
        3
    );
    assert_eq!(
        state.map.rooms[&sickbay_id]
            .hazards
            .iter()
            .filter(|&&h| h == HazardType::Fire)
            .count(),
        1
    );
}
