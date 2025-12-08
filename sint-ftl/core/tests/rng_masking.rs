use sint_core::{Action, GameLogic, GamePhase, ItemType};

#[test]
fn test_simulation_masks_rng_outcome() {
    let mut state = GameLogic::new_game(vec!["P1".to_string()], 12345);
    state.phase = GamePhase::TacticalPlanning;

    // Setup: P1 in Cannons (8), Has Ammo.
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = 8;
        p.inventory.push(ItemType::Peppernut);
        p.ap = 2;
    }

    let initial_hp = state.enemy.hp;

    // 1. Propose Action (Shoot)
    state.proposal_queue.push(sint_core::ProposedAction {
        id: "test".to_string(),
        player_id: "P1".to_string(),
        action: Action::Shoot,
    });

    // 2. "Simulate" by projecting the state
    // This is what get_valid_actions does internally to validate subsequent moves.
    let mut projected_state = state.clone();
    sint_core::logic::resolution::resolve_proposal_queue(&mut projected_state, true); // true = simulation

    // 3. Check Projected Outcome
    println!(
        "Initial HP: {}, Projected HP: {}",
        initial_hp, projected_state.enemy.hp
    );

    // Assert that we CANNOT see the result.
    // HP should be UNCHANGED in projection (RNG Masked).
    assert_eq!(
        projected_state.enemy.hp, initial_hp,
        "HP should NOT change in simulation"
    );

    // But Ammo should still be consumed (Deterministic cost).
    assert!(
        projected_state.players["P1"].inventory.is_empty(),
        "Ammo consumed in projection"
    );
}
