use sint_core::{logic::GameLogic, types::*};

fn create_test_game() -> GameState {
    let player_ids = vec!["P1".to_string(), "P2".to_string()];
    GameLogic::new_game(player_ids, 12345)
}

#[test]
fn test_boss_defeat_triggers_rest() {
    let mut state = create_test_game();

    // Setup: Boss at 1 HP
    state.enemy.hp = 1;
    state.phase = GamePhase::TacticalPlanning;

    // Give P1 ammo and put in Cannons
    let p1 = state.players.get_mut("P1").unwrap();
    p1.room_id = 6; // Cannons
    p1.inventory.push(ItemType::Peppernut);

    // Action: Shoot
    let action = Action::Game(GameAction::Shoot);

    // Force hit (rng seed manipulation or mock? Simulation ignores RNG for hit check usually, but execute uses it)
    // We can just set enemy HP to 0 manually to simulate the kill effect if needed,
    // but let's try to run the action.
    // To guarantee hit, we might need to manipulate RNG or try multiple times.
    // Easier: Just force the state transition that happens ON kill.

    // Let's actually execute the action.
    let result = GameLogic::apply_action(state.clone(), "P1", action, None);
    assert!(result.is_ok());
    let mut state = result.unwrap();

    // Check if enemy is defeated (Shoot handler logic)
    // Note: RNG might cause a miss. Let's force HP to 0 to test the Phase transition logic specifically.
    state.enemy.hp = 0;
    // Manually trigger the logic that would happen if shoot hit (state update)
    state.enemy.state = EnemyState::Defeated;

    // Now Advance Phase (Tactical -> Execution -> EnemyAction)
    // To trigger advance_phase from Tactical, players must VoteReady.
    let action = Action::Game(GameAction::VoteReady { ready: true });
    let result = GameLogic::apply_action(state.clone(), "P1", action.clone(), None); // P1 Ready
    let state = result.unwrap();
    let result = GameLogic::apply_action(state.clone(), "P2", action, None); // P2 Ready
    let mut state = result.unwrap();

    // Now we should be in Execution.
    assert_eq!(state.phase, GamePhase::Execution);

    // Advance Execution -> EnemyAction (Requires expending AP or Pass)
    state.players.get_mut("P1").unwrap().ap = 0;
    state.players.get_mut("P2").unwrap().ap = 0;

    // Force advance (VoteReady in Execution triggers advance if AP=0)
    state.players.get_mut("P1").unwrap().is_ready = false;
    state.players.get_mut("P2").unwrap().is_ready = false;

    // To advance from Execution when AP is 0, we can just call Pass or VoteReady?
    // In Execution, apply_game_action doesn't support Pass. But VoteReady is allowed.
    // And advance_phase checks if ready.
    let action = Action::Game(GameAction::VoteReady { ready: true });
    let state_res = GameLogic::apply_action(state.clone(), "P1", action.clone(), None);
    let state = state_res.unwrap();
    let state_res = GameLogic::apply_action(state.clone(), "P2", action, None);
    let mut state = state_res.unwrap();

    // Now we should be in EnemyAction (which auto-advances to MorningReport usually)
    // Wait, `advance_phase` logic for Execution -> EnemyAction is checked inside `apply_game_action`
    // when players VoteReady? Yes.

    assert_eq!(state.phase, GamePhase::EnemyAction);

    // Now advance from EnemyAction -> MorningReport
    state.players.get_mut("P1").unwrap().is_ready = false;
    state.players.get_mut("P2").unwrap().is_ready = false;
    let action = Action::Game(GameAction::VoteReady { ready: true });
    let state_res = GameLogic::apply_action(state, "P1", action.clone(), None);
    let state = state_res.unwrap();
    let state_res = GameLogic::apply_action(state, "P2", action, None);
    let mut state = state_res.unwrap();

    // Logic check: EnemyAction should detect Defeated and set is_resting
    assert!(state.is_resting, "Should be resting after boss defeat");
    assert_eq!(
        state.phase,
        GamePhase::MorningReport,
        "Should transition to MorningReport"
    );

    // Verify MorningReport skips card draw (Deck size shouldn't change? Or latest event is None?)
    let deck_size_before = state.deck.len();

    // Advance MorningReport -> EnemyTelegraph
    state.players.get_mut("P1").unwrap().is_ready = false;
    state.players.get_mut("P2").unwrap().is_ready = false;
    let action = Action::Game(GameAction::VoteReady { ready: true });
    let state_res = GameLogic::apply_action(state, "P1", action.clone(), None);
    let state = state_res.unwrap();
    let state_res = GameLogic::apply_action(state, "P2", action, None);
    let state = state_res.unwrap();

    assert_eq!(state.phase, GamePhase::EnemyTelegraph);
    assert_eq!(
        state.deck.len(),
        deck_size_before,
        "No card should be drawn during rest"
    );
    assert!(
        state.enemy.next_attack.is_none(),
        "No telegraph during rest"
    );
}

#[test]
fn test_rest_round_end_spawns_boss() {
    let mut state = create_test_game();
    state.is_resting = true;
    state.enemy.state = EnemyState::Defeated;
    state.enemy.hp = 0;
    state.phase = GamePhase::Execution; // End of Rest Round Execution

    // Advance Execution -> EnemyAction
    state.players.get_mut("P1").unwrap().ap = 0;
    state.players.get_mut("P2").unwrap().ap = 0;

    let action = Action::Game(GameAction::VoteReady { ready: true });
    let state = GameLogic::apply_action(state, "P1", action.clone(), None).unwrap();
    let mut state = GameLogic::apply_action(state, "P2", action, None).unwrap();

    assert_eq!(state.phase, GamePhase::EnemyAction);

    // Advance EnemyAction -> MorningReport (Should Spawn Boss)
    state.players.get_mut("P1").unwrap().is_ready = false;
    state.players.get_mut("P2").unwrap().is_ready = false;
    let action = Action::Game(GameAction::VoteReady { ready: true });
    let state = GameLogic::apply_action(state, "P1", action.clone(), None).unwrap();
    let state = GameLogic::apply_action(state, "P2", action, None).unwrap();

    assert!(!state.is_resting, "Rest should be over");
    assert_eq!(state.boss_level, 1, "Boss level should increment");
    assert_eq!(
        state.enemy.state,
        EnemyState::Active,
        "New boss should be active"
    );
    assert!(state.enemy.hp > 0, "New boss should have HP");
}

#[test]
fn test_hull_repair_in_cargo() {
    let mut state = create_test_game();
    state.phase = GamePhase::TacticalPlanning;
    state.hull_integrity = 15;

    let p1_id = "P1".to_string();
    // Move P1 to Cargo (3)
    state.players.get_mut(&p1_id).unwrap().room_id = 3;

    // Action: Repair
    let action = Action::Game(GameAction::Repair);
    let result = GameLogic::apply_action(state.clone(), &p1_id, action, None);

    assert!(result.is_ok());
    let state = result.unwrap();

    // Action is queued, not executed. Advance to Execution.
    let action = Action::Game(GameAction::VoteReady { ready: true });
    let state_res = GameLogic::apply_action(state, "P1", action.clone(), None);
    let state = state_res.unwrap();
    let state_res = GameLogic::apply_action(state, "P2", action, None);
    let state = state_res.unwrap();

    assert_eq!(state.phase, GamePhase::Execution);
    assert_eq!(
        state.hull_integrity, 16,
        "Hull should increase by 1 after execution"
    );
}

#[test]
fn test_hull_repair_limit() {
    let mut state = create_test_game();
    state.phase = GamePhase::TacticalPlanning;
    state.hull_integrity = MAX_HULL;

    let p1_id = "P1".to_string();
    state.players.get_mut(&p1_id).unwrap().room_id = 3;

    let action = Action::Game(GameAction::Repair);
    let result = GameLogic::apply_action(state, &p1_id, action, None);

    assert!(result.is_err(), "Should not allow repair at max hull");
}
