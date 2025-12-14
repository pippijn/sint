use proptest::prelude::*;
use rand::prelude::*;
use sint_core::logic::actions::get_valid_actions;
use sint_core::{GameLogic, logic::find_room_with_system_in_map, types::*};
use sint_solver::scoring::rl::{RlScoringWeights, score_rl};

#[test]
fn test_resource_gathering_is_net_positive() {
    let player_ids = vec!["P1".to_owned()];
    let weights = RlScoringWeights::default();
    let mut state = GameLogic::new_game(player_ids.clone(), 42);

    // Setup: Room 1 has a Peppernut, P1 is in Room 1
    state.phase = GamePhase::TacticalPlanning;
    let room_id = 1;
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = room_id;
        p.ap = 10;
        p.inventory.clear();
    }
    if let Some(room) = state.map.rooms.get_mut(&room_id) {
        room.items.push(ItemType::Peppernut);
    }

    let pickup_act = GameAction::PickUp {
        item_type: ItemType::Peppernut,
    };
    let history = [(PlayerId::from("P1"), pickup_act.clone())];
    let borrowed_history: Vec<_> = history.iter().collect();

    let mut next_state =
        GameLogic::apply_action(state.clone(), "P1", Action::Game(pickup_act), None).unwrap();
    // Resolve to apply pickup
    sint_core::logic::resolution::resolve_proposal_queue(&mut next_state, false);

    let details = score_rl(&state, &next_state, &borrowed_history, &weights);

    println!(
        "Pickup Reward: {}, Logistics: {}, Progression: {}, Vitals: {}",
        details.total, details.logistics, details.progression, details.vitals
    );
    assert!(
        details.total > 0.0,
        "Resource gathering should be net-positive, got {}",
        details.total
    );
}

#[test]
fn test_action_variance_in_complex_state() {
    let player_ids = vec!["P1".to_owned()];
    let weights = RlScoringWeights::default();
    let mut state = GameLogic::new_game(player_ids.clone(), 42);

    // Setup: Room 1 has Fire, Water, and a Peppernut. P1 is in Room 1.
    state.phase = GamePhase::TacticalPlanning;
    let room_id = 1;
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = room_id;
        p.ap = 10;
        p.inventory.clear();
    }
    if let Some(room) = state.map.rooms.get_mut(&room_id) {
        room.hazards.push(HazardType::Fire);
        room.hazards.push(HazardType::Water);
        room.items.push(ItemType::Peppernut);
        room.system = Some(SystemType::Kitchen); // Allow Bake
    }

    let valid_actions = get_valid_actions(&state, "P1");
    let mut rewards = Vec::new();

    for action in valid_actions {
        if let Action::Game(ga) = action {
            let mut next_state =
                GameLogic::apply_action(state.clone(), "P1", Action::Game(ga.clone()), None)
                    .unwrap();
            sint_core::logic::resolution::resolve_proposal_queue(&mut next_state, false);

            let history = [(PlayerId::from("P1"), ga)];
            let borrowed_history: Vec<_> = history.iter().collect();
            let reward = score_rl(&state, &next_state, &borrowed_history, &weights).total;
            rewards.push(reward);
        }
    }

    let unique_rewards: std::collections::HashSet<_> =
        rewards.iter().map(|r| format!("{:.4}", r)).collect();
    println!("Unique rewards: {:?}", unique_rewards);

    assert!(
        unique_rewards.len() >= 3,
        "Reward function is too sparse, only {} distinct signals",
        unique_rewards.len()
    );
}

#[test]
fn test_defense_discovery_signal() {
    let player_ids = vec!["P1".to_owned()];
    let weights = RlScoringWeights::default();
    let mut state = GameLogic::new_game(player_ids.clone(), 42);

    // Find Bridge room
    let bridge_room =
        find_room_with_system_in_map(&state.map, SystemType::Bridge).expect("Bridge not found");

    state.phase = GamePhase::TacticalPlanning;
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = bridge_room;
        p.ap = 10;
    }
    state.enemy.next_attack = Some(EnemyAttack {
        target_room: Some(bridge_room),
        target_system: Some(SystemType::Bridge),
        effect: AttackEffect::Fireball,
    });

    let raise_shields = GameAction::RaiseShields;
    let mut next_state_shields = GameLogic::apply_action(
        state.clone(),
        "P1",
        Action::Game(raise_shields.clone()),
        None,
    )
    .unwrap();
    sint_core::logic::resolution::resolve_proposal_queue(&mut next_state_shields, false);

    let history_shields = [(PlayerId::from("P1"), raise_shields)];
    let reward_shields = score_rl(
        &state,
        &next_state_shields,
        &history_shields.iter().collect::<Vec<_>>(),
        &weights,
    )
    .total;

    let pass = GameAction::Pass;
    let mut next_state_pass =
        GameLogic::apply_action(state.clone(), "P1", Action::Game(pass.clone()), None).unwrap();
    sint_core::logic::resolution::resolve_proposal_queue(&mut next_state_pass, false);

    let history_pass = [(PlayerId::from("P1"), pass)];
    let reward_pass = score_rl(
        &state,
        &next_state_pass,
        &history_pass.iter().collect::<Vec<_>>(),
        &weights,
    )
    .total;

    println!(
        "Shields Reward: {}, Pass Reward: {}",
        reward_shields, reward_pass
    );
    assert!(
        reward_shields > reward_pass,
        "Raising shields when targeted should be better than passing. Shields: {}, Pass: {}",
        reward_shields,
        reward_pass
    );
}

#[test]
fn test_extinguish_vs_move_neutral() {
    let player_ids = vec!["P1".to_owned()];
    let weights = RlScoringWeights::default();
    let mut state = GameLogic::new_game(player_ids.clone(), 42);

    state.phase = GamePhase::TacticalPlanning;
    let room_id = 1;
    if let Some(p) = state.players.get_mut("P1") {
        p.room_id = room_id;
        p.ap = 10;
    }
    if let Some(room) = state.map.rooms.get_mut(&room_id) {
        room.hazards.push(HazardType::Fire);
    }

    let extinguish = GameAction::Extinguish;
    let mut next_state_ext =
        GameLogic::apply_action(state.clone(), "P1", Action::Game(extinguish.clone()), None)
            .unwrap();
    sint_core::logic::resolution::resolve_proposal_queue(&mut next_state_ext, false);
    let history_ext = [(PlayerId::from("P1"), extinguish)];
    let reward_ext = score_rl(
        &state,
        &next_state_ext,
        &history_ext.iter().collect::<Vec<_>>(),
        &weights,
    )
    .total;

    let move_act = GameAction::Move { to_room: 0 };
    let mut next_state_move =
        GameLogic::apply_action(state.clone(), "P1", Action::Game(move_act.clone()), None).unwrap();
    sint_core::logic::resolution::resolve_proposal_queue(&mut next_state_move, false);
    let history_move = [(PlayerId::from("P1"), move_act)];
    let reward_move = score_rl(
        &state,
        &next_state_move,
        &history_move.iter().collect::<Vec<_>>(),
        &weights,
    )
    .total;

    println!(
        "Extinguish Reward: {}, Move Reward: {}",
        reward_ext, reward_move
    );
    assert!(
        reward_ext > reward_move,
        "Extinguishing hazard should be better than moving away"
    );
}

proptest! {
    #[test]
    fn test_random_productive_actions_better_than_pass(seed in 0u64..100) {
        let player_ids = vec!["P1".to_owned()];
        let weights = RlScoringWeights::default();
        let mut state = GameLogic::new_game(player_ids.clone(), seed);
        state.phase = GamePhase::TacticalPlanning;

        let p_id = "P1";
        if let Some(p) = state.players.get_mut(p_id) {
            p.ap = 10;
        }

        // Add some hazards and items randomly
        let mut rng = StdRng::seed_from_u64(seed);
        for room in state.map.rooms.values_mut() {
            if rng.random_bool(0.3) { room.hazards.push(HazardType::Fire); }
            if rng.random_bool(0.3) { room.hazards.push(HazardType::Water); }
            if rng.random_bool(0.3) { room.items.push(ItemType::Peppernut); }
        }

        let valid_actions = get_valid_actions(&state, p_id);

        let pass = GameAction::Pass;
        let mut next_state_pass = GameLogic::apply_action(state.clone(), p_id, Action::Game(pass.clone()), None).unwrap();
        sint_core::logic::resolution::resolve_proposal_queue(&mut next_state_pass, false);
        let history_pass = [(PlayerId::from(p_id), pass)];
        let reward_pass = score_rl(&state, &next_state_pass, &history_pass.iter().collect::<Vec<_>>(), &weights).total;

        for action in valid_actions {
            if let Action::Game(ga) = action {
                if matches!(ga, GameAction::Pass | GameAction::Chat { .. } | GameAction::VoteReady { .. }) { continue; }

                let mut next_state = GameLogic::apply_action(state.clone(), p_id, Action::Game(ga.clone()), None).unwrap();
                sint_core::logic::resolution::resolve_proposal_queue(&mut next_state, false);

                let history_ga = [(PlayerId::from(p_id), ga.clone())];
                let reward = score_rl(&state, &next_state, &history_ga.iter().collect::<Vec<_>>(), &weights).total;

                // 1. Sanity check: no massive penalties for valid actions
                prop_assert!(reward > -5.0, "Action {:?} resulted in very low reward: {}", ga, reward);

                // 2. Productive actions should be better than passing
                if matches!(ga, GameAction::Extinguish | GameAction::Repair | GameAction::Shoot) {
                    prop_assert!(reward > reward_pass, "Productive action {:?} (reward {}) should be better than passing (reward {})", ga, reward, reward_pass);
                }
            }
        }
    }
}
