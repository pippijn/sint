use sint_core::{Action, GameLogic, GamePhase, ItemType};

#[test]
fn test_boss_progression() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::Execution;

    // Check Initial Boss (Level 0)
    assert_eq!(state.boss_level, 0);
    assert_eq!(state.enemy.name, "The Petty Thief");

    // Give P1 ammo and put in Cannons
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sint_core::logic::ROOM_CANNONS;
        p.inventory = vec![ItemType::Peppernut; 100]; // Infinite ammo
        p.ap = 0;
        p.is_ready = true;
    }

    // Force Boss HP to 1
    state.enemy.hp = 1;

    let mut attempts = 0;
    while state.boss_level == 0 && attempts < 100 {
        // Queue Shot
        state.proposal_queue.push(sint_core::ProposedAction {
            id: "kill_shot".to_string(),
            player_id: "P1".to_string(),
            action: Action::Shoot,
        });

        // Ensure HP is 1
        state.enemy.hp = 1;

        sint_core::logic::resolution::resolve_proposal_queue(&mut state, false);
        attempts += 1;
    }

    assert!(state.boss_level == 1, "Should progress to Level 1");
    assert_eq!(state.enemy.name, "The Monster");
    assert_eq!(state.enemy.hp, 10);
}

#[test]
fn test_victory_condition() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::Execution;

    // Jump to Level 3 (The Kraken)
    state.boss_level = 3;
    state.enemy = sint_core::logic::get_boss(3);
    assert_eq!(state.enemy.name, "The Kraken");

    // Set Kraken HP to 1
    state.enemy.hp = 1;

    // P1 Shoot setup
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = sint_core::logic::ROOM_CANNONS;
        p.inventory = vec![ItemType::Peppernut; 100];
    }

    // Kill Kraken
    let mut attempts = 0;
    while state.phase != GamePhase::Victory && attempts < 100 {
        state.proposal_queue.push(sint_core::ProposedAction {
            id: "kill_shot".to_string(),
            player_id: "P1".to_string(),
            action: Action::Shoot,
        });
        state.enemy.hp = 1;
        sint_core::logic::resolution::resolve_proposal_queue(&mut state, false);
        attempts += 1;
    }

    assert_eq!(state.phase, GamePhase::Victory);
}
