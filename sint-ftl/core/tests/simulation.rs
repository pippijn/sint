use sint_core::{
    logic::GameLogic,
    types::{Action, Card, CardId, CardSolution, CardType, GameAction, GamePhase, SystemType},
    GameError,
};

#[test]
fn test_interact_validation_wrong_room() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // 1. Add a dummy situation that requires being in the Bridge (9)
    let card = Card {
        id: CardId::TheBook, // Requires Bridge
        title: "Test Card".to_string(),
        description: "Go to Bridge".to_string(),
        card_type: CardType::Situation,
        options: vec![],
        solution: Some(CardSolution {
            room_id: Some(SystemType::Bridge.as_u32()), // Room 9
            ap_cost: 1,
            item_cost: None,
            required_players: 1,
        }),
    };
    state.active_situations.push(card);

    // 2. Place Player in Kitchen (6)
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = SystemType::Kitchen.as_u32();
        p.ap = 2;
    }

    // 3. Attempt Interact -> Should Fail
    let res = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(GameAction::Interact),
        None,
    );

    match res {
        Ok(_) => panic!("Interact should have failed due to wrong room"),
        Err(GameError::InvalidAction(msg)) => {
            assert_eq!(msg, "Nothing to Interact with here");
        }
        Err(e) => panic!("Wrong error type: {:?}", e),
    }

    // 4. Move to Bridge -> Should Succeed
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = SystemType::Bridge.as_u32();
    }
    let res_ok = GameLogic::apply_action(state, "P1", Action::Game(GameAction::Interact), None);
    assert!(res_ok.is_ok(), "Interact should succeed in correct room");
}

#[test]
fn test_shoot_simulation_no_side_effects() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Setup: P1 in Cannons (8) with Ammo
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = SystemType::Cannons.as_u32();
        p.inventory.push(sint_core::types::ItemType::Peppernut);
        p.ap = 2;
    }

    let initial_hp = state.enemy.hp;

    // 1. Queue a Shot (This runs projection validation internally)
    let res = GameLogic::apply_action(state.clone(), "P1", Action::Game(GameAction::Shoot), None);
    assert!(res.is_ok());
    let state_queued = res.unwrap();

    let queued_rng = state_queued.rng_seed;

    // 2. Verify Projection (Simulation) didn't touch RNG or HP in the *returned* state
    // Note: apply_action returns state with the action in the queue.
    // It calls `resolve_proposal_queue(..., true)` on a *clone* to validate.
    // So the state returned (state_queued) should NOT have executed the shot yet.
    // However, we want to ensure the *logic* inside the handler respects the simulation flag.

    // To properly test the handler's simulation logic, we can manually call resolve on a clone with simulation=true
    let mut sim_state = state_queued.clone();

    // The queue has the "Shoot" action.
    // resolve_proposal_queue(..., true) is called.
    sint_core::logic::resolution::resolve_proposal_queue(&mut sim_state, true);

    // 3. Assertions on Simulated State
    // In simulation:
    // - Ammo should be consumed (Movement/Inventory logic usually runs in sim to allow chaining)
    // - BUT RNG should NOT advance (to avoid oracle)
    // - AND Enemy HP should NOT change (to avoid oracle)

    let p1_sim = sim_state.players.get("P1").unwrap();
    assert!(
        p1_sim.inventory.is_empty(),
        "Simulation should consume ammo to validate subsequent actions"
    );

    assert_eq!(
        sim_state.rng_seed, queued_rng,
        "Simulation should NOT advance RNG seed"
    );
    assert_eq!(
        sim_state.enemy.hp, initial_hp,
        "Simulation should NOT damage enemy"
    );

    // 4. Now Execute for Real (simulation=false)
    let mut exec_state = state_queued.clone();
    // Simulate Phase Transition to Execution which calls resolve(false)
    sint_core::logic::resolution::resolve_proposal_queue(&mut exec_state, false);

    // In Execution:
    // - RNG SHOULD advance (a roll happened)
    // - Enemy HP MIGHT change (depending on hit/miss, but for this test we just check RNG changed)
    assert_ne!(
        exec_state.rng_seed, queued_rng,
        "Execution SHOULD advance RNG seed"
    );
}
