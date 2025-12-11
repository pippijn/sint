use sint_core::{
    types::{Action, CardId, CardSolution, CardType, GameAction, GamePhase, ItemType}, GameLogic,
};

fn new_test_game(players: Vec<String>) -> sint_core::types::GameState {
    let mut state = GameLogic::new_game(players, 12345);
    state.deck.clear(); // Remove RNG events for deterministic unit tests
    state
}

#[test]
fn test_costume_party() {
    let mut state = new_test_game(vec!["P1".to_string(), "P2".to_string(), "P3".to_string()]);
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

    use sint_core::logic::cards::get_behavior;
    let behavior = get_behavior(CardId::CostumeParty);
    behavior.on_activate(&mut state);

    assert_eq!(state.players["P1"].room_id, 6);
    assert_eq!(state.players["P2"].room_id, 8);
    assert_eq!(state.players["P3"].room_id, 3);
}

#[test]
fn test_amerigo_eats_peppernuts() {
    let mut state = new_test_game(vec!["P1".to_string()]);
    state.phase = GamePhase::MorningReport;

    let storage_id = sint_core::logic::find_room_with_system_in_map(
        &state.map,
        sint_core::types::SystemType::Storage,
    )
    .unwrap();

    // Place Peppernuts in Storage
    if let Some(r) = state.map.rooms.get_mut(&storage_id) {
        r.items.push(ItemType::Peppernut);
        r.items.push(ItemType::Peppernut);
    }

    use sint_core::logic::cards::get_behavior;
    let behavior = get_behavior(CardId::Amerigo);
    behavior.on_round_end(&mut state);

    // Should eat 1
    // Note: Storage starts with 5 items by default + 2 added here = 7. Eating 1 -> 6.
    assert_eq!(state.map.rooms[&storage_id].items.len(), 6);
}

#[test]
fn test_afternoon_nap_blocks_actions() {
    let mut state = new_test_game(vec!["P1".to_string(), "P2".to_string()]);
    state.phase = GamePhase::TacticalPlanning;

    let card = sint_core::types::Card {
        id: CardId::AfternoonNap,
        title: "Nap".to_string(),
        description: "Reader sleeps".to_string(),
        card_type: sint_core::types::CardType::Situation,
        options: vec![],
        solution: None,
    };
    state.active_situations.push(card);

    let kitchen = sint_core::logic::find_room_with_system_in_map(
        &state.map,
        sint_core::types::SystemType::Kitchen,
    )
    .unwrap();

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
    let mut state = new_test_game(vec!["P1".to_string()]);
    state.phase = GamePhase::TacticalPlanning;

    let card = sint_core::types::Card {
        id: CardId::WailingAlarm,
        title: "Alarm".to_string(),
        description: "No Shields".to_string(),
        card_type: sint_core::types::CardType::Situation,
        options: vec![],
        solution: None,
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
    let mut state = new_test_game(vec!["P1".to_string()]);
    state.phase = GamePhase::MorningReport;

    let card = sint_core::types::Card {
        id: CardId::MonsterDough,
        title: "Dough".to_string(),
        description: "Boom 3 turns".to_string(),
        card_type: sint_core::types::CardType::Timebomb { rounds_left: 1 },
        options: vec![],
        solution: None,
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
    if let sint_core::types::CardType::Timebomb { rounds_left } = card.card_type {
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
    let mut state = new_test_game(vec!["P1".to_string()]);
    state.phase = GamePhase::MorningReport;

    let card = sint_core::types::Card {
        id: CardId::FluWave,
        title: "Flu".to_string(),
        description: "AP -1".to_string(),
        card_type: sint_core::types::CardType::Timebomb { rounds_left: 1 },
        options: vec![],
        solution: None,
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
    let mut state = new_test_game(vec!["P1".to_string(), "P2".to_string()]);
    state.phase = GamePhase::MorningReport;

    if let Some(p) = state.players.get_mut("P1") {
        p.inventory.push(ItemType::Extinguisher);
    }
    if let Some(p) = state.players.get_mut("P2") {
        p.inventory.push(ItemType::Wheelbarrow);
    }

    use sint_core::logic::cards::get_behavior;
    let behavior = get_behavior(CardId::LuckyDip);
    behavior.on_activate(&mut state);

    assert_eq!(state.players["P1"].inventory[0], ItemType::Wheelbarrow);
    assert_eq!(state.players["P2"].inventory[0], ItemType::Extinguisher);
}

#[test]
fn test_man_overboard_death() {
    let mut state = new_test_game(vec!["P1".to_string()]);
    state.phase = GamePhase::MorningReport;

    let card = sint_core::types::Card {
        id: CardId::ManOverboard,
        title: "Man Overboard".to_string(),
        description: "Die".to_string(),
        card_type: sint_core::types::CardType::Timebomb { rounds_left: 1 },
        options: vec![],
        solution: None,
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
    let mut state = new_test_game(vec!["P1".to_string()]);
    state.phase = GamePhase::MorningReport;

    let card = sint_core::types::Card {
        id: CardId::MicePlague,
        title: "Mice".to_string(),
        description: "Eat nuts".to_string(),
        card_type: sint_core::types::CardType::Situation,
        options: vec![],
        solution: None,
    };
    state.active_situations.push(card);

    let storage = sint_core::logic::find_room_with_system_in_map(
        &state.map,
        sint_core::types::SystemType::Storage,
    )
    .unwrap();

    if let Some(r) = state.map.rooms.get_mut(&storage) {
        r.items.clear();
        r.items.push(ItemType::Peppernut);
        r.items.push(ItemType::Peppernut);
        r.items.push(ItemType::Peppernut);
    }

    use sint_core::logic::cards::get_behavior;
    let behavior = get_behavior(CardId::MicePlague);
    behavior.on_round_end(&mut state);

    assert_eq!(state.map.rooms[&storage].items.len(), 1);
}

#[test]
fn test_overheating_ap_loss() {
    let mut state = new_test_game(vec!["P1".to_string()]);
    state.phase = GamePhase::MorningReport;

    let card = sint_core::types::Card {
        id: CardId::Overheating,
        title: "Heat".to_string(),
        description: "AP -1 if in Engine".to_string(),
        card_type: sint_core::types::CardType::Situation,
        options: vec![],
        solution: None,
    };
    state.active_situations.push(card);

    let engine = sint_core::logic::find_room_with_system_in_map(
        &state.map,
        sint_core::types::SystemType::Engine,
    )
    .unwrap();

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = engine;
    }

    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 2; // Reset
    }

    use sint_core::logic::cards::get_behavior;
    let behavior = get_behavior(CardId::Overheating);
    behavior.on_round_start(&mut state);

    assert_eq!(state.players["P1"].ap, 1);
}

#[test]
fn test_panic_move() {
    let mut state = new_test_game(vec!["P1".to_string()]);
    let bridge = sint_core::logic::find_room_with_system_in_map(
        &state.map,
        sint_core::types::SystemType::Bridge,
    )
    .unwrap();
    let dorm = sint_core::logic::find_room_with_system_in_map(
        &state.map,
        sint_core::types::SystemType::Dormitory,
    )
    .unwrap();

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = bridge;
    }

    use sint_core::logic::cards::get_behavior;
    let behavior = get_behavior(CardId::Panic);
    behavior.on_activate(&mut state);

    assert_eq!(state.players["P1"].room_id, dorm);
}

#[test]
fn test_rudderless_damage() {
    let mut state = new_test_game(vec!["P1".to_string()]);
    let card = sint_core::types::Card {
        id: CardId::Rudderless,
        title: "Rudderless".to_string(),
        description: "+1 Dmg".to_string(),
        card_type: sint_core::types::CardType::Situation,
        options: vec![],
        solution: None,
    };
    state.active_situations.push(card);

    use sint_core::logic::cards::get_behavior;
    let behavior = get_behavior(CardId::Rudderless);
    assert_eq!(behavior.get_hazard_modifier(&state), 1);
}

#[test]
fn test_seagull_attack_block_move() {
    let mut state = new_test_game(vec!["P1".to_string()]);
    state.phase = GamePhase::TacticalPlanning;

    let card = sint_core::types::Card {
        id: CardId::SeagullAttack,
        title: "Seagull".to_string(),
        description: "No move with nut".to_string(),
        card_type: sint_core::types::CardType::Situation,
        options: vec![],
        solution: None,
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
fn test_seasick_restriction() {}

#[test]
fn test_shoe_setting_skip_turn() {
    let mut state = new_test_game(vec!["P1".to_string()]);

    let card = sint_core::types::Card {
        id: CardId::ShoeSetting,
        title: "Shoe".to_string(),
        description: "Skip turn".to_string(),
        card_type: sint_core::types::CardType::Timebomb { rounds_left: 0 },
        options: vec![],
        solution: None,
    };
    state.active_situations.push(card);

    use sint_core::logic::cards::get_behavior;
    let behavior = get_behavior(CardId::ShoeSetting);

    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 2;
    }
    behavior.on_round_start(&mut state);

    assert_eq!(state.players["P1"].ap, 0);
    assert!(state
        .active_situations
        .iter()
        .all(|c| c.id != CardId::ShoeSetting));
}

#[test]
fn test_short_circuit() {
    let mut state = new_test_game(vec!["P1".to_string()]);

    use sint_core::logic::cards::get_behavior;
    let behavior = get_behavior(CardId::ShortCircuit);
    behavior.on_activate(&mut state);

    let engine_id = sint_core::logic::find_room_with_system_in_map(
        &state.map,
        sint_core::types::SystemType::Engine,
    )
    .unwrap();
    let engine = state.map.rooms.get(&engine_id).unwrap();
    assert!(engine.hazards.contains(&sint_core::types::HazardType::Fire));
}

#[test]
fn test_silent_force() {
    let mut state = new_test_game(vec!["P1".to_string()]);
    if let Some(r) = state.map.rooms.get_mut(&5) {
        r.hazards.push(sint_core::types::HazardType::Fire);
    }
    if let Some(r) = state.map.rooms.get_mut(&6) {
        r.hazards.push(sint_core::types::HazardType::Water);
    }

    use sint_core::logic::cards::get_behavior;
    let behavior = get_behavior(CardId::SilentForce);
    behavior.on_activate(&mut state);

    assert!(state.map.rooms[&5].hazards.is_empty());
    assert!(state.map.rooms[&6].hazards.is_empty());
}

#[test]
fn test_sing_a_song() {}

#[test]
fn test_slippery_deck() {
    let mut state = new_test_game(vec!["P1".to_string()]);
    let card = sint_core::types::Card {
        id: CardId::SlipperyDeck,
        title: "Slippery".to_string(),
        description: "Move 0, Act +1".to_string(),
        card_type: sint_core::types::CardType::Situation,
        options: vec![],
        solution: None,
    };
    state.active_situations.push(card);

    let cost_move =
        sint_core::logic::actions::action_cost(&state, "P1", &GameAction::Move { to_room: 0 });
    assert_eq!(cost_move, 0);

    let cost_bake = sint_core::logic::actions::action_cost(&state, "P1", &GameAction::Bake);
    assert_eq!(cost_bake, 2);
}

#[test]
fn test_sticky_floor() {
    let mut state = new_test_game(vec!["P1".to_string()]);
    let card = sint_core::types::Card {
        id: CardId::StickyFloor,
        title: "Sticky".to_string(),
        description: "Move to Kitchen +1".to_string(),
        card_type: sint_core::types::CardType::Situation,
        options: vec![],
        solution: None,
    };
    state.active_situations.push(card);

    let kitchen = sint_core::logic::find_room_with_system_in_map(
        &state.map,
        sint_core::types::SystemType::Kitchen,
    )
    .unwrap();
    let bridge = sint_core::logic::find_room_with_system_in_map(
        &state.map,
        sint_core::types::SystemType::Bridge,
    )
    .unwrap();

    let cost_kitchen = sint_core::logic::actions::action_cost(
        &state,
        "P1",
        &GameAction::Move { to_room: kitchen },
    );
    assert_eq!(cost_kitchen, 2);

    let cost_other =
        sint_core::logic::actions::action_cost(&state, "P1", &GameAction::Move { to_room: bridge });
    assert_eq!(cost_other, 1);
}

#[test]
fn test_sugar_rush() {
    let mut state = new_test_game(vec!["P1".to_string()]);
    let card = sint_core::types::Card {
        id: CardId::SugarRush,
        title: "Sugar".to_string(),
        description: "Move 0, No Shoot".to_string(),
        card_type: sint_core::types::CardType::Situation,
        options: vec![],
        solution: None,
    };
    state.active_situations.push(card);

    let cost_move =
        sint_core::logic::actions::action_cost(&state, "P1", &GameAction::Move { to_room: 0 });
    assert_eq!(cost_move, 0);

    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 8;
    }
    let res = GameLogic::apply_action(state, "P1", Action::Game(GameAction::Shoot), None);
    assert!(res.is_err());
}

#[test]
fn test_can_solve_wailing_alarm_logic() {
    let mut state = new_test_game(vec!["P1".to_string()]);
    state.phase = GamePhase::TacticalPlanning;

    // Add Wailing Alarm
    let card = sint_core::types::Card {
        id: CardId::WailingAlarm,
        title: "Alarm".to_string(),
        description: "Test".to_string(),
        card_type: CardType::Situation,
        options: vec![],
        solution: Some(CardSolution {
            target_system: None, // Any room
            ap_cost: 1,
            item_cost: None,
            required_players: 1,
        }),
    };
    state.active_situations.push(card);

    // 1. P1 in Kitchen (System 6) -> Should Fail
    let kitchen = sint_core::logic::find_room_with_system_in_map(
        &state.map,
        sint_core::types::SystemType::Kitchen,
    )
    .unwrap();
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
    let mut state = new_test_game(vec!["P1".to_string()]);
    state.phase = GamePhase::TacticalPlanning;

    // Add Amerigo (Requires Storage)
    let card = sint_core::logic::cards::get_behavior(CardId::Amerigo).get_struct();
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
    let storage = sint_core::logic::find_room_with_system_in_map(
        &state.map,
        sint_core::types::SystemType::Storage,
    )
    .unwrap();
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
fn test_sugar_rush_blocks_shoot_but_allows_solve() {
    let mut state = new_test_game(vec!["P1".to_string()]);
    state.phase = GamePhase::TacticalPlanning;

    // Add Sugar Rush (Requires Kitchen, Blocks Shoot)
    let card = sint_core::logic::cards::get_behavior(CardId::SugarRush).get_struct();
    state.active_situations.push(card);

    let kitchen = sint_core::logic::find_room_with_system_in_map(
        &state.map,
        sint_core::types::SystemType::Kitchen,
    )
    .unwrap();
    let cannons = sint_core::logic::find_room_with_system_in_map(
        &state.map,
        sint_core::types::SystemType::Cannons,
    )
    .unwrap();

    // 1. P1 in Kitchen -> Can Interact (Solve)
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = kitchen;
        p.ap = 2;
    }
    let res_solve = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Interact),
        None,
    );
    assert!(res_solve.is_ok(), "Sugar Rush should be solvable in Kitchen");

    // 2. P1 in Cannons -> Cannot Shoot (Blocked by validation)
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = cannons;
        p.inventory.push(sint_core::types::ItemType::Peppernut);
    }
    let res_shoot = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Shoot),
        None,
    );
    
    assert!(res_shoot.is_err(), "Sugar Rush should block Shoot");
    if let Err(sint_core::GameError::InvalidAction(msg)) = res_shoot {
        assert!(msg.contains("Too shaky"), "Error should be specific to Sugar Rush");
    } else {
        panic!("Wrong error type");
    }
}
