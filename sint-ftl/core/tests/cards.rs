use sint_core::{
    GameError, GameLogic,
    logic::{
        actions::action_cost, cards::get_behavior, find_room_with_system_in_map,
        resolution::process_round_end,
    },
    types::*,
};

fn new_test_game(players: Vec<String>) -> GameState {
    let mut state = GameLogic::new_game(players, 12345);
    state.deck.clear(); // Remove RNG events for deterministic unit tests
    state
}

#[test]
fn test_costume_party() {
    let mut state = new_test_game(vec!["P1".to_owned(), "P2".to_owned(), "P3".to_owned()]);
    state.phase = GamePhase::MorningReport;

    // Move players to specific spots (manually for setup)
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 3;
    } // Dorm
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = 6;
    } // Kitchen
    if let Some(p) = state.players.get_mut("P3") {
        p.room_id = 8;
    } // Cannons

    let behavior = get_behavior(CardId::CostumeParty);
    behavior.on_activate(&mut state);

    assert_eq!(state.players["P1"].room_id, 6);
    assert_eq!(state.players["P2"].room_id, 8);
    assert_eq!(state.players["P3"].room_id, 3);
}

#[test]
fn test_amerigo_eats_peppernuts() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    state.phase = GamePhase::MorningReport;

    let storage_id = find_room_with_system_in_map(&state.map, SystemType::Storage).unwrap();

    // Place Peppernuts in Storage
    if let Some(r) = state.map.rooms.get_mut(&storage_id) {
        r.items.push(ItemType::Peppernut);
        r.items.push(ItemType::Peppernut);
    }

    let behavior = get_behavior(CardId::Amerigo);
    state.active_situations.push(behavior.get_struct());
    process_round_end(&mut state);

    // Should eat 1
    // Note: Storage starts with 5 items by default + 2 added here = 7. Eating 1 -> 6.
    assert_eq!(state.map.rooms[&storage_id].items.len(), 6);
}

#[test]
fn test_afternoon_nap_blocks_actions() {
    let mut state = new_test_game(vec!["P1".to_owned(), "P2".to_owned()]);
    state.phase = GamePhase::TacticalPlanning;

    let card = Card {
        id: CardId::AfternoonNap,
        title: "Nap".to_owned(),
        description: "Reader sleeps".to_owned(),
        card_type: CardType::Situation,
        options: vec![].into(),
        solution: None,
        affected_player: Some("P1".to_owned()),
    };
    state.active_situations.push(card);

    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = kitchen;
    }

    let res = GameLogic::apply_action(state.clone(), "P1", Action::Game(GameAction::Bake), None);
    assert!(res.is_err(), "P1 is Reader/Asleep, cannot act");

    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = kitchen;
    }
    let res2 = GameLogic::apply_action(state.clone(), "P2", Action::Game(GameAction::Bake), None);
    assert!(res2.is_ok(), "P2 is not Reader, can act");
}

#[test]
fn test_wailing_alarm_blocks_bonus_actions() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    state.phase = GamePhase::TacticalPlanning;

    let card = Card {
        id: CardId::WailingAlarm,
        title: "Alarm".to_owned(),
        description: "No Shields".to_owned(),
        card_type: CardType::Situation,
        options: vec![].into(),
        solution: None,
        affected_player: None,
    };
    state.active_situations.push(card);

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 5;
        p.ap = 2;
    }

    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::RaiseShields),
        None,
    );
    assert!(res.is_err(), "Alarm blocks shields");
}

#[test]
fn test_monster_dough_trigger() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    state.phase = GamePhase::MorningReport;

    let card = Card {
        id: CardId::MonsterDough,
        title: "Dough".to_owned(),
        description: "Boom 3 turns".to_owned(),
        card_type: CardType::Timebomb { rounds_left: 1 },
        options: vec![].into(),
        solution: None,
        affected_player: None,
    };
    state.active_situations.push(card);

    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap(); // M -> T
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap(); // T -> P
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap(); // P -> E
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 0;
    }
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap(); // E -> EA
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap(); // EA -> M

    let card = state
        .active_situations
        .iter()
        .find(|c| c.id == CardId::MonsterDough)
        .unwrap();
    if let CardType::Timebomb { rounds_left } = card.card_type {
        assert_eq!(rounds_left, 0);
    }

    state.phase = GamePhase::TacticalPlanning;
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 6;
        p.ap = 2;
    }
    let res = GameLogic::apply_action(state, "P1", Action::Game(GameAction::Bake), None);
    assert!(res.is_err(), "Monster Dough exploded, Kitchen blocked");
}

#[test]
fn test_flu_wave_ap_reduction() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    state.phase = GamePhase::MorningReport;

    let card = Card {
        id: CardId::FluWave,
        title: "Flu".to_owned(),
        description: "AP -1".to_owned(),
        card_type: CardType::Timebomb { rounds_left: 1 },
        options: vec![].into(),
        solution: None,
        affected_player: None,
    };
    state.active_situations.push(card);

    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap(); // M -> T
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap(); // T -> P
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap(); // P -> E
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 0;
    }
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap(); // E -> EA

    // EA -> M
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    assert_eq!(state.players["P1"].ap, 1, "Flu Wave should reduce AP to 1");
    assert!(state.active_situations.is_empty());
}

#[test]
fn test_lucky_dip_swap() {
    let mut state = new_test_game(vec!["P1".to_owned(), "P2".to_owned()]);
    state.phase = GamePhase::MorningReport;

    if let Some(p) = state.players.get_mut("P1") {
        p.inventory.push(ItemType::Extinguisher);
    }
    if let Some(p) = state.players.get_mut("P2") {
        p.inventory.push(ItemType::Wheelbarrow);
    }

    let behavior = get_behavior(CardId::LuckyDip);
    behavior.on_activate(&mut state);

    assert_eq!(state.players["P1"].inventory[0], ItemType::Wheelbarrow);
    assert_eq!(state.players["P2"].inventory[0], ItemType::Extinguisher);
}

#[test]
fn test_man_overboard_death() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    state.phase = GamePhase::MorningReport;

    let card = Card {
        id: CardId::ManOverboard,
        title: "Man Overboard".to_owned(),
        description: "Die".to_owned(),
        card_type: CardType::Timebomb { rounds_left: 1 },
        options: vec![].into(),
        solution: None,
        affected_player: None,
    };
    state.active_situations.push(card);

    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 0;
    }
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::VoteReady { ready: true }),
        None,
    )
    .unwrap();

    assert!(!state.players.contains_key("P1"));
}

#[test]
fn test_mice_plague_eats_nuts() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    state.phase = GamePhase::MorningReport;

    let storage = find_room_with_system_in_map(&state.map, SystemType::Storage).unwrap();

    if let Some(r) = state.map.rooms.get_mut(&storage) {
        r.items.clear();
        r.items.push(ItemType::Peppernut);
        r.items.push(ItemType::Peppernut);
        r.items.push(ItemType::Peppernut);
    }

    let behavior = get_behavior(CardId::MicePlague);
    state.active_situations.push(behavior.get_struct());
    process_round_end(&mut state);

    assert_eq!(state.map.rooms[&storage].items.len(), 1);
}

#[test]
fn test_overheating_ap_loss() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    state.phase = GamePhase::MorningReport;

    let card = Card {
        id: CardId::Overheating,
        title: "Heat".to_owned(),
        description: "AP -1 if in Engine".to_owned(),
        card_type: CardType::Situation,
        options: vec![].into(),
        solution: None,
        affected_player: None,
    };
    state.active_situations.push(card);

    let engine = find_room_with_system_in_map(&state.map, SystemType::Engine).unwrap();

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = engine;
    }

    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 2; // Reset
    }

    let behavior = get_behavior(CardId::Overheating);
    behavior.on_round_start(&mut state);

    assert_eq!(state.players["P1"].ap, 1);
}

#[test]
fn test_panic_move() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    let bridge = find_room_with_system_in_map(&state.map, SystemType::Bridge).unwrap();
    let dorm = find_room_with_system_in_map(&state.map, SystemType::Dormitory).unwrap();

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = bridge;
    }

    let behavior = get_behavior(CardId::Panic);
    behavior.on_activate(&mut state);

    assert_eq!(state.players["P1"].room_id, dorm);
}

#[test]
fn test_rudderless_damage() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    let card = Card {
        id: CardId::Rudderless,
        title: "Rudderless".to_owned(),
        description: "+1 Dmg".to_owned(),
        card_type: CardType::Situation,
        options: vec![].into(),
        solution: None,
        affected_player: None,
    };
    state.active_situations.push(card);

    let behavior = get_behavior(CardId::Rudderless);
    assert_eq!(behavior.get_hazard_modifier(&state), 1);
}

#[test]
fn test_seagull_attack_block_move() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    state.phase = GamePhase::TacticalPlanning;

    let card = Card {
        id: CardId::SeagullAttack,
        title: "Seagull".to_owned(),
        description: "No move with nut".to_owned(),
        card_type: CardType::Situation,
        options: vec![].into(),
        solution: None,
        affected_player: None,
    };
    state.active_situations.push(card);

    if let Some(p) = state.players.get_mut("P1") {
        p.inventory.push(ItemType::Peppernut);
        p.room_id = 7;
    }

    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Move { to_room: 6 }),
        None,
    );
    assert!(res.is_err());

    state.players.get_mut("P1").unwrap().inventory.clear();
    let res = GameLogic::apply_action(
        state,
        "P1",
        Action::Game(GameAction::Move { to_room: 6 }),
        None,
    );
    assert!(res.is_ok());
}

#[test]
fn test_seasick_restriction() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    state.phase = GamePhase::TacticalPlanning;

    let card = Card {
        id: CardId::Seasick,
        title: "Seasick".to_owned(),
        description: "Either Walk or Act".to_owned(),
        card_type: CardType::Situation,
        options: vec![].into(),
        solution: None,
        affected_player: None,
    };
    state.active_situations.push(card);

    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();
    let bow = find_room_with_system_in_map(&state.map, SystemType::Bow).unwrap();

    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 2;
        p.room_id = 0; // Hallway
    }

    // 1. Move then Bake -> Should Fail
    let state_after_move = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Move { to_room: bow }),
        None,
    )
    .unwrap();
    let res = GameLogic::apply_action(state_after_move, "P1", Action::Game(GameAction::Bake), None);
    assert!(res.is_err(), "Seasick should block Bake after Move");

    // 2. Bake then Move -> Should Fail
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = kitchen;
    }
    let state_after_bake =
        GameLogic::apply_action(state.clone(), "P1", Action::Game(GameAction::Bake), None).unwrap();
    let res = GameLogic::apply_action(
        state_after_bake,
        "P1",
        Action::Game(GameAction::Move { to_room: 0 }),
        None,
    );
    assert!(res.is_err(), "Seasick should block Move after Bake");
}

#[test]
fn test_anchor_stuck_required_players() {
    let mut state = new_test_game(vec!["P1".to_owned(), "P2".to_owned(), "P3".to_owned()]);
    state.phase = GamePhase::TacticalPlanning;

    let card = get_behavior(CardId::AnchorStuck).get_struct();
    state.active_situations.push(card);

    let bow = find_room_with_system_in_map(&state.map, SystemType::Bow).unwrap();

    // 1. Only P1 at Bow -> Should Fail
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = bow;
        p.ap = 2;
    }
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = 0;
    }
    if let Some(p) = state.players.get_mut("P3") {
        p.room_id = 0;
    }

    let res1 = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Interact),
        None,
    );
    assert!(
        res1.is_err(),
        "Anchor Stuck should require 3 players, but only 1 is present"
    );

    // 2. P1 and P2 at Bow -> Should Fail
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = bow;
    }
    let res2 = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Interact),
        None,
    );
    assert!(
        res2.is_err(),
        "Anchor Stuck should require 3 players, but only 2 are present"
    );

    // 3. P1, P2, and P3 at Bow -> Should Succeed
    if let Some(p) = state.players.get_mut("P3") {
        p.room_id = bow;
    }
    let res3 = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Interact),
        None,
    );
    assert!(
        res3.is_ok(),
        "Anchor Stuck should succeed with 3 players present"
    );
}

#[test]
fn test_shoe_setting_skip_turn() {
    let mut state = new_test_game(vec!["P1".to_owned()]);

    let card = Card {
        id: CardId::ShoeSetting,
        title: "Shoe".to_owned(),
        description: "Skip turn".to_owned(),
        card_type: CardType::Timebomb { rounds_left: 0 },
        options: vec![].into(),
        solution: None,
        affected_player: None,
    };
    state.active_situations.push(card);

    let behavior = get_behavior(CardId::ShoeSetting);

    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 2;
    }
    behavior.on_round_start(&mut state);

    assert_eq!(state.players["P1"].ap, 0);
    assert!(
        state
            .active_situations
            .iter()
            .all(|c| c.id != CardId::ShoeSetting)
    );
}

#[test]
fn test_short_circuit() {
    let mut state = new_test_game(vec!["P1".to_owned()]);

    let behavior = get_behavior(CardId::ShortCircuit);
    behavior.on_activate(&mut state);

    let engine_id = find_room_with_system_in_map(&state.map, SystemType::Engine).unwrap();
    let engine = state.map.rooms.get(&engine_id).unwrap();
    assert!(engine.hazards.contains(&HazardType::Fire));
}

#[test]
fn test_silent_force() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    if let Some(r) = state.map.rooms.get_mut(&5) {
        r.hazards.push(HazardType::Fire);
    }
    if let Some(r) = state.map.rooms.get_mut(&6) {
        r.hazards.push(HazardType::Water);
    }

    let behavior = get_behavior(CardId::SilentForce);
    behavior.on_activate(&mut state);

    assert!(state.map.rooms[&5].hazards.is_empty());
    assert!(state.map.rooms[&6].hazards.is_empty());
}

#[test]
fn test_sing_a_song() {}

#[test]
fn test_slippery_deck() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    let card = Card {
        id: CardId::SlipperyDeck,
        title: "Slippery".to_owned(),
        description: "Move 0, Act +1".to_owned(),
        card_type: CardType::Situation,
        options: vec![].into(),
        solution: None,
        affected_player: None,
    };
    state.active_situations.push(card);

    let cost_move = action_cost(&state, "P1", &GameAction::Move { to_room: 0 });
    assert_eq!(cost_move, 0);

    let cost_bake = action_cost(&state, "P1", &GameAction::Bake);
    assert_eq!(cost_bake, 2);
}

#[test]
fn test_sticky_floor() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    let card = Card {
        id: CardId::StickyFloor,
        title: "Sticky".to_owned(),
        description: "Move to Kitchen +1".to_owned(),
        card_type: CardType::Situation,
        options: vec![].into(),
        solution: None,
        affected_player: None,
    };
    state.active_situations.push(card);

    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();
    let bridge = find_room_with_system_in_map(&state.map, SystemType::Bridge).unwrap();

    let cost_kitchen = action_cost(&state, "P1", &GameAction::Move { to_room: kitchen });
    assert_eq!(cost_kitchen, 2);

    let cost_other = action_cost(&state, "P1", &GameAction::Move { to_room: bridge });
    assert_eq!(cost_other, 1);
}

#[test]
fn test_sugar_rush() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    let card = Card {
        id: CardId::SugarRush,
        title: "Sugar".to_owned(),
        description: "Move 0, No Shoot".to_owned(),
        card_type: CardType::Situation,
        options: vec![].into(),
        solution: None,
        affected_player: None,
    };
    state.active_situations.push(card);

    let cost_move = action_cost(&state, "P1", &GameAction::Move { to_room: 0 });
    assert_eq!(cost_move, 0);

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = find_room_with_system_in_map(&state.map, SystemType::Cannons).unwrap();
    }
    let res = GameLogic::apply_action(state, "P1", Action::Game(GameAction::Shoot), None);
    assert!(res.is_err());
}

#[test]
fn test_can_solve_wailing_alarm_logic() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    state.phase = GamePhase::TacticalPlanning;

    // Add Wailing Alarm
    let card = Card {
        id: CardId::WailingAlarm,
        title: "Alarm".to_owned(),
        description: "Test".to_owned(),
        card_type: CardType::Situation,
        options: vec![].into(),
        solution: Some(CardSolution {
            target_system: None, // Any room
            ap_cost: 1,
            item_cost: None,
            required_players: 1,
        }),
        affected_player: None,
    };
    state.active_situations.push(card);

    // 1. P1 in Kitchen (System 6) -> Should Fail
    let kitchen = find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = kitchen;
        p.ap = 2;
    }

    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Interact),
        None,
    );
    // InteractHandler uses can_solve. WailingAlarm's can_solve checks for Empty Room.
    assert!(res.is_err(), "Wailing Alarm should fail in Kitchen");

    // 2. P1 in Hallway (System None) -> Should Succeed
    let hallway = 0;
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = hallway;
    }

    let res_ok = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Interact),
        None,
    );
    assert!(res_ok.is_ok(), "Wailing Alarm should succeed in Hallway");
}

#[test]
fn test_default_can_solve_logic_amerigo() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    state.phase = GamePhase::TacticalPlanning;

    // Add Amerigo (Requires Storage)
    let card = get_behavior(CardId::Amerigo).get_struct();
    state.active_situations.push(card);

    // 1. P1 in Hallway -> Fail
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 0;
        p.ap = 2;
    }
    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Interact),
        None,
    );
    assert!(res.is_err(), "Amerigo should fail in Hallway");

    // 2. P1 in Storage -> Succeed
    let storage = find_room_with_system_in_map(&state.map, SystemType::Storage).unwrap();
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = storage;
    }
    let res_ok = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Interact),
        None,
    );
    assert!(res_ok.is_ok(), "Amerigo should succeed in Storage");
}

#[test]
fn test_afternoon_nap_rotation() {
    let mut state = new_test_game(vec!["P1".to_owned(), "P2".to_owned(), "P3".to_owned()]);
    let behavior = get_behavior(CardId::AfternoonNap);

    // Turn 1 -> P1 (Index 0)
    state.turn_count = 1;
    let card = behavior.get_struct();
    state.active_situations.push(card);
    behavior.on_activate(&mut state);
    assert_eq!(
        state.active_situations.last().unwrap().affected_player,
        Some("P1".to_owned())
    );

    // Turn 2 -> P2 (Index 1)
    state.turn_count = 2;
    let card = behavior.get_struct();
    state.active_situations.push(card);
    behavior.on_activate(&mut state);
    assert_eq!(
        state.active_situations.last().unwrap().affected_player,
        Some("P2".to_owned())
    );

    // Turn 4 -> P1 (Index 0, wrapped)
    state.turn_count = 4;
    let card = behavior.get_struct();
    state.active_situations.push(card);
    behavior.on_activate(&mut state);
    assert_eq!(
        state.active_situations.last().unwrap().affected_player,
        Some("P1".to_owned())
    );
}

#[test]
fn test_afternoon_nap_persistence() {
    let mut state = new_test_game(vec!["P1".to_owned(), "P2".to_owned()]);
    state.phase = GamePhase::TacticalPlanning;
    let behavior = get_behavior(CardId::AfternoonNap);

    // Activate on Turn 1 (Targets P1)
    state.turn_count = 1;
    let card = behavior.get_struct();
    state.active_situations.push(card);
    behavior.on_activate(&mut state);

    // Advance Turn to 2 (Rotation would target P2, but card should persist P1)
    state.turn_count = 2;

    // Check P1 (Should be blocked)
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 5; // Kitchen
    }
    let res_p1 = GameLogic::apply_action(state.clone(), "P1", Action::Game(GameAction::Bake), None);
    assert!(
        res_p1.is_err(),
        "P1 should remain the Reader despite turn advance"
    );

    // Check P2 (Should be free)
    if let Some(p) = state.players.get_mut("P2") {
        p.room_id = 5; // Kitchen
    }
    let res_p2 = GameLogic::apply_action(state.clone(), "P2", Action::Game(GameAction::Bake), None);
    match res_p2 {
        Ok(_) => {}
        Err(e) => panic!("P2 should not be affected but failed with: {:?}", e),
    }
}

#[test]
fn test_afternoon_nap_error_message() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    state.phase = GamePhase::TacticalPlanning;
    if let Some(p) = state.players.get_mut("P1") {
        p.name = "Captain".to_owned();
        p.room_id = 6;
    }

    let behavior = get_behavior(CardId::AfternoonNap);

    // Activate
    state.turn_count = 1;
    let card = behavior.get_struct();
    state.active_situations.push(card);
    behavior.on_activate(&mut state);

    // Try action
    let res = GameLogic::apply_action(state, "P1", Action::Game(GameAction::Bake), None);

    match res {
        Err(GameError::InvalidAction(msg)) => {
            assert!(
                msg.contains("The Reader (Captain)"),
                "Error message should contain player name. Got: {}",
                msg
            );
        }
        _ => panic!("Expected InvalidAction error"),
    }
}

#[test]
fn test_anchor_loose() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    state.phase = GamePhase::MorningReport;

    // Remove all hazards
    for room in state.map.rooms.values_mut() {
        room.hazards.clear();
    }

    let behavior = get_behavior(CardId::AnchorLoose);
    behavior.on_round_start(&mut state);

    // One room should now have Water
    let water_count: usize = state
        .map
        .rooms
        .values()
        .map(|r| {
            r.hazards
                .iter()
                .filter(|&&h| h == HazardType::Water)
                .count()
        })
        .sum();

    assert_eq!(water_count, 1);
}

#[test]

fn test_afternoon_nap_multiple_players() {
    let mut state = new_test_game(vec!["P1".to_owned(), "P2".to_owned()]);

    state.phase = GamePhase::TacticalPlanning;

    // Card 1: P1 is asleep

    let card1 = Card {
        id: CardId::AfternoonNap,

        title: "Nap 1".to_owned(),

        description: "P1 sleeps".to_owned(),

        card_type: CardType::Situation,

        options: vec![].into(),

        solution: None,

        affected_player: Some("P1".to_owned()),
    };

    // Card 2: P2 is asleep

    let card2 = Card {
        id: CardId::AfternoonNap,

        title: "Nap 2".to_owned(),

        description: "P2 sleeps".to_owned(),

        card_type: CardType::Situation,

        options: vec![].into(),

        solution: None,

        affected_player: Some("P2".to_owned()),
    };

    state.active_situations.push(card1);

    state.active_situations.push(card2);

    let res1 = GameLogic::apply_action(state.clone(), "P1", Action::Game(GameAction::Bake), None);

    assert!(res1.is_err(), "P1 should be blocked by Card 1");

    let res2 = GameLogic::apply_action(state.clone(), "P2", Action::Game(GameAction::Bake), None);

    assert!(res2.is_err(), "P2 should be blocked by Card 2");
}

#[test]

fn test_golden_nut_triggers_rest_round() {
    let mut state = new_test_game(vec!["P1".to_owned()]);

    state.phase = GamePhase::TacticalPlanning;

    state.enemy.hp = 1;

    state.boss_level = 0;

    let behavior = get_behavior(CardId::GoldenNut);

    behavior.on_solved(&mut state);

    assert_eq!(state.enemy.hp, 0);

    assert_eq!(state.enemy.state, EnemyState::Defeated);

    assert_eq!(
        state.boss_level, 0,
        "Boss level should not increase until rest round ends"
    );
}

#[test]
fn test_blockade_adjacency_requirement() {
    let mut state = new_test_game(vec!["P1".to_owned()]);
    state.phase = GamePhase::TacticalPlanning;

    let card = get_behavior(CardId::Blockade).get_struct();
    state.active_situations.push(card);

    let cannons_id = find_room_with_system_in_map(&state.map, SystemType::Cannons).unwrap();
    let hallway = 0; // Adjacent to Cannons in Star layout

    // 1. P1 in Dormitory (Not adjacent) -> Fail
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 2; // Dormitory
        p.ap = 2;
    }
    let res1 = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Interact),
        None,
    );
    assert!(
        res1.is_err(),
        "Blockade should not be solvable from Dormitory"
    );

    // 2. P1 in Hallway (Adjacent) -> Succeed
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = hallway;
    }
    // Verify hallway is indeed adjacent to cannons
    assert!(state.map.rooms[&hallway].neighbors.contains(&cannons_id));

    let res2 = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Interact),
        None,
    );
    assert!(res2.is_ok(), "Blockade should be solvable from Hallway");
}
