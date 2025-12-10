use sint_core::logic::cards::get_behavior;
use sint_core::HazardType;
use sint_core::ItemType;
use sint_core::{Action, CardId, CardType, GameLogic, GamePhase};

#[test]
fn test_card_slippery_deck() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Inject C04
    let card = state
        .deck
        .iter()
        .find(|c| c.id == CardId::SlipperyDeck)
        .cloned()
        .unwrap();
    state.active_situations.push(card);

    // Setup P1
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sint_core::types::SystemType::Kitchen.as_u32(); // Kitchen
        p.ap = 2;
    }

    // 1. Move should be FREE (0 AP)
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Move {
            to_room: sint_core::types::SystemType::Hallway.as_u32(),
        },
        None,
    )
    .unwrap();
    assert_eq!(
        state.players["P1"].ap, 2,
        "Move should cost 0 AP with Slippery Deck"
    );

    // 2. Bake should cost +1 (Total 2 AP)
    // Bake base cost 1 -> +1 = 2.
    // P1 currently in 6 (Kitchen), Moving to 7 (Hallway).
    // Can't bake in Hallway.
    // Let's Move BACK to 6 (Free) then Bake.
    state = GameLogic::apply_action(
        state,
        "P1",
        Action::Move {
            to_room: sint_core::types::SystemType::Kitchen.as_u32(),
        },
        None,
    )
    .unwrap();
    assert_eq!(state.players["P1"].ap, 2);

    state = GameLogic::apply_action(state, "P1", Action::Bake, None).unwrap();
    assert_eq!(state.players["P1"].ap, 0, "Bake should cost 2 AP (1+1)");
}

#[test]
fn test_card_mutiny_explosion() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);

    // Inject C11 Mutiny with 1 round left
    let mut card = state
        .deck
        .iter()
        .find(|c| c.id == CardId::Mutiny)
        .cloned()
        .unwrap();
    card.card_type = CardType::Timebomb { rounds_left: 1 };
    state.active_situations.push(card);

    let initial_hull = state.hull_integrity;

    // Trigger Round End (Transition Execution -> EnemyAction)
    state.phase = GamePhase::Execution;
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 0;
        p.is_ready = true;
    }
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();

    assert_eq!(state.phase, GamePhase::EnemyAction);

    // Mutiny triggers at End of Round (EnemyAction -> MorningReport)
    // Wait, let's check `logic/cards/impls/c11_mutiny.rs`: `on_round_end`.
    // `advance_phase` calls `on_round_end` when exiting `EnemyAction`.

    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::MorningReport);

    // Should have exploded (-10 Hull)
    assert_eq!(
        state.hull_integrity,
        initial_hull - 10,
        "Mutiny should deal 10 hull damage"
    );
    assert!(
        state
            .active_situations
            .iter()
            .all(|c| c.id != CardId::Mutiny),
        "Mutiny card should be removed"
    );
}

#[test]
fn test_card_sing_a_song() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);

    // Setup Hazards
    state
        .map
        .rooms
        .get_mut(&sint_core::types::SystemType::Kitchen.as_u32())
        .unwrap()
        .hazards
        .push(HazardType::Fire);
    state
        .map
        .rooms
        .get_mut(&sint_core::types::SystemType::Cargo.as_u32())
        .unwrap()
        .hazards
        .push(HazardType::Water);

    // Get C21
    let card = state
        .deck
        .iter()
        .find(|c| c.id == CardId::SingASong)
        .cloned()
        .unwrap();

    // Simulate Draw (Manual activation)
    get_behavior(card.id).on_activate(&mut state);

    // Verify hazards removed
    assert!(
        state.map.rooms[&sint_core::types::SystemType::Kitchen.as_u32()]
            .hazards
            .is_empty(),
        "Fire should be removed"
    );
    assert!(
        state.map.rooms[&sint_core::types::SystemType::Cargo.as_u32()]
            .hazards
            .is_empty(),
        "Water should be removed"
    );
}

#[test]
fn test_flu_wave_effect_application() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);

    // 1. Inject Flu Wave with 1 round left
    let mut card = state
        .deck
        .iter()
        .find(|c| c.id == CardId::FluWave)
        .cloned()
        .unwrap();
    state.deck.clear(); // Isolate test from random draws

    card.card_type = CardType::Timebomb { rounds_left: 1 };
    state.active_situations.push(card);

    // 2. Advance Phase: Planning -> Execution -> EnemyAction -> MorningReport
    state.phase = GamePhase::Execution;
    // Set P1 AP to 0 so we advance
    if let Some(p) = state.players.get_mut("P1") {
        p.ap = 0;
        p.is_ready = true;
    }

    // Execution -> EnemyAction
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::EnemyAction);

    // EnemyAction -> MorningReport (This triggers `on_round_end` then `on_round_start`)
    // `on_round_end` (in EnemyAction) ticks timer to 0.
    // `advance_phase` resets AP to 2.
    // `on_round_start` (in MorningReport) sees timer 0, sets AP to 1, removes card.
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::MorningReport);

    // 3. Verify AP is 1
    // Wait, AP is reset in MorningReport? No, in my recent edit:
    // "Reset AP (New Round)" is at start of EnemyAction block (transition FROM Execution TO EnemyAction?).
    // No, `advance_phase` matches CURRENT phase.
    // If state.phase is `Execution` and we call `advance_phase`:
    // It goes to `EnemyAction`.
    // My previous edit inserted `p.ap = 2` in `GamePhase::EnemyAction` block.
    // `match state.phase { GamePhase::EnemyAction => { ... } }` runs when we are IN EnemyAction and transitioning OUT.
    // So AP reset happens when moving EnemyAction -> MorningReport.

    // Let's trace carefully:
    // P1 votes ready in Execution. `advance_phase` called. `match Execution` -> `state.phase = EnemyAction`.
    // P1 votes ready in EnemyAction. `advance_phase` called. `match EnemyAction`:
    //   -> Reset AP to 2.
    //   -> Call `on_round_end` (Flu timer 1->0).
    //   -> Call `on_round_start` (Flu sees 0, AP=1, remove card).
    //   -> state.phase = MorningReport.

    assert_eq!(state.players["P1"].ap, 1, "Flu Wave should reduce AP to 1");

    // 4. Verify Card Removed
    assert!(
        !state
            .active_situations
            .iter()
            .any(|c| c.id == CardId::FluWave),
        "Flu Wave should be removed after triggering"
    );

    // 5. Advance to Planning
    // MorningReport -> EnemyTelegraph -> TacticalPlanning.
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap(); // To Telegraph
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap(); // To Planning

    // Verify AP persists (since Telegraph no longer resets it)
    assert_eq!(
        state.players["P1"].ap, 1,
        "AP Penalty should persist to Planning"
    );

    // 6. Complete this round (Planning -> Execution)
    // Spend the 1 AP to Pass (or just Pass)
    state = GameLogic::apply_action(state, "P1", Action::Pass, None).unwrap();
    assert_eq!(state.phase, GamePhase::Execution);

    // Execution -> EnemyAction
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::EnemyAction);

    // 7. Next Round Start (EnemyAction -> MorningReport)
    // This should reset AP to 2. FluWave is gone, so no penalty.
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::MorningReport);

    // MorningReport -> Telegraph -> Planning
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    state = GameLogic::apply_action(state, "P1", Action::VoteReady { ready: true }, None).unwrap();
    assert_eq!(state.phase, GamePhase::TacticalPlanning);

    // 8. Verify AP restored to 2
    assert_eq!(
        state.players["P1"].ap, 2,
        "AP should be restored to 2 in next round"
    );
}

#[test]
fn test_seagull_attack_validation_in_planning() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Inject Seagull Attack
    let seagull_card = state
        .deck
        .iter()
        .find(|c| c.id == CardId::SeagullAttack)
        .cloned()
        .unwrap();
    state.active_situations.push(seagull_card);

    // Give P1 a Peppernut
    use sint_core::ItemType;
    if let Some(p) = state.players.get_mut("P1") {
        p.inventory.push(ItemType::Peppernut);
        p.room_id = 7; // Hallway
    }

    // Attempt Move
    let res = GameLogic::apply_action(state, "P1", Action::Move { to_room: 6 }, None);

    assert!(
        res.is_err(),
        "Should be blocked by Seagull Attack validation"
    );
    if let Err(e) = res {
        assert!(
            e.to_string().contains("Seagull Attack"),
            "Error should mention Seagull Attack"
        );
    }
}

#[test]
fn test_seagull_attack_prevents_move_after_pickup() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Inject
    let seagull_card = state
        .deck
        .iter()
        .find(|c| c.id == CardId::SeagullAttack)
        .cloned()
        .unwrap();
    state.active_situations.push(seagull_card);

    // P1 in Room 11 (Storage, has nuts)
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sint_core::types::SystemType::Storage.as_u32();
        p.ap = 2;
    }

    // 1. Queue PickUp (Cost 1 AP)
    let state = GameLogic::apply_action(
        state,
        "P1",
        Action::PickUp {
            item_type: ItemType::Peppernut,
        },
        None,
    )
    .unwrap();

    // 2. Queue Move (Cost 1 AP)
    // This should now FAIL because projected state has Peppernut
    let res = GameLogic::apply_action(state, "P1", Action::Move { to_room: 7 }, None);

    assert!(
        res.is_err(),
        "Move should be blocked by Seagull Attack (Fail Early)"
    );
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("Cannot move while holding Peppernuts"));
}

#[test]
fn test_slippery_and_listing_stacking() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Inject SlipperyDeck (Move=0, Actions=+1)
    let c1 = state
        .deck
        .iter()
        .find(|c| c.id == CardId::SlipperyDeck)
        .cloned()
        .unwrap();
    // Inject Listing (Move=0, Actions=*2)
    let c2 = state
        .deck
        .iter()
        .find(|c| c.id == CardId::Listing)
        .cloned()
        .unwrap();

    // Case A: Slippery THEN Listing
    state.active_situations = vec![c1.clone(), c2.clone()];

    // Move: 1 -> Slip(0) -> List(0). Cost 0.
    let cost_move =
        sint_core::logic::actions::action_cost(&state, "P1", &Action::Move { to_room: 0 });
    assert_eq!(cost_move, 0, "Move should be free (0 -> 0)");

    // Bake: 1 -> Slip(2) -> List(4). Cost 4.
    let cost_bake = sint_core::logic::actions::action_cost(&state, "P1", &Action::Bake);
    assert_eq!(cost_bake, 4, "Bake: 1 -> +1(2) -> *2(4)");

    // Case B: Listing THEN Slippery
    state.active_situations = vec![c2.clone(), c1.clone()];

    // Move: 1 -> List(0) -> Slip(0). Cost 0.
    let cost_move =
        sint_core::logic::actions::action_cost(&state, "P1", &Action::Move { to_room: 0 });
    assert_eq!(cost_move, 0, "Move should be free");

    // Bake: 1 -> List(2) -> Slip(3). Cost 3.
    let cost_bake = sint_core::logic::actions::action_cost(&state, "P1", &Action::Bake);
    assert_eq!(cost_bake, 3, "Bake: 1 -> *2(2) -> +1(3)");
}

#[test]
fn test_serialization_roundtrip() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    // Mutate state
    state.phase = GamePhase::Execution;
    // Add a card
    state.active_situations.push(state.deck[0].clone());
    // Damage player
    if let Some(p) = state.players.get_mut("P1") {
        p.hp = 1;
    }

    // Serialize
    let json = serde_json::to_string(&state).unwrap();

    // Deserialize
    let state2: sint_core::GameState = serde_json::from_str(&json).unwrap();

    // Compare
    assert_eq!(state, state2, "State should match after roundtrip");
}
