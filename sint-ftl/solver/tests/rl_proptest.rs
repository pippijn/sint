use proptest::prelude::*;
use rand::prelude::*;
use sint_core::logic::actions::get_valid_actions;
use sint_core::{GameLogic, types::*};
use sint_solver::scoring::rl::{RlScoringWeights, score_rl};

proptest! {
    /// Test that repetitive "empty" loops (juggling items, moving back and forth)
    /// always result in negative total reward, even in deep/chaotic states.
    #[test]
    fn test_no_reward_hacking_loops(seed in 0u64..u64::MAX, steps in 0usize..2000) {
        let player_ids = vec!["P1".to_owned()];
        let weights = RlScoringWeights::default();
        let mut state = GameLogic::new_game(player_ids.clone(), seed);
        let mut rng = StdRng::seed_from_u64(seed);

        // 1. Advance to a random "deep" state
        for _ in 0..steps {
            if state.phase == GamePhase::GameOver || state.phase == GamePhase::Victory { break; }
            let valid = get_valid_actions(&state, "P1");
            if valid.is_empty() { break; }
            let act = valid.choose(&mut rng).unwrap().clone();
            if let Ok(next) = GameLogic::apply_action(state.clone(), "P1", act, None) {
                state = next;
                sint_core::logic::resolution::resolve_proposal_queue(&mut state, false);
            }
        }

        // Ensure state is at least TacticalPlanning for testing if it ended in Lobby
        if state.phase == GamePhase::Lobby || state.phase == GamePhase::Setup || state.phase == GamePhase::MorningReport {
             state.phase = GamePhase::TacticalPlanning;
        }

        // 2. Perturb state to ensure we hit critical boundaries
        state.hull_integrity = rng.random_range(1..21);
        if state.enemy.max_hp > 0 {
            state.enemy.hp = rng.random_range(1..state.enemy.max_hp + 1);
        }
        state.phase = GamePhase::TacticalPlanning;
        state.active_situations.clear();

        // Ensure player exists and is healthy for testing
        if state.players.get("P1").is_none() {
            let p = Player {
                id: "P1".to_owned(),
                name: "The Reader".to_owned(),
                room_id: 1,
                hp: 3,
                ap: 10,
                inventory: Default::default(),
                status: Default::default(),
                is_ready: false,
            };
            state.players.insert(p);
        } else {
            let p = state.players.get_mut("P1").unwrap();
            p.hp = 3;
            p.ap = 10;
            p.status.clear();
            p.is_ready = false;
            p.inventory.clear();
        }

        // --- Loop Tests ---
        let mut current_state = state.clone();
        let mut total_juggling_reward = 0.0;
        let room_id = current_state.players.get("P1").unwrap().room_id;
        current_state.map.rooms.get_mut(&room_id).unwrap().items.push(ItemType::Peppernut);

        // Pickup
        let pickup_act = GameAction::PickUp { item_type: ItemType::Peppernut };
        let history = vec![(PlayerId::from("P1"), pickup_act.clone())];
        let borrowed_history: Vec<_> = history.iter().collect();
        let next_state = GameLogic::apply_action(current_state.clone(), "P1", Action::Game(pickup_act), None).unwrap();
        total_juggling_reward += score_rl(&current_state, &next_state, &borrowed_history, &weights).total;
        current_state = next_state;

        // Drop
        let drop_act = GameAction::Drop { item_index: 0 };
        let history = vec![(PlayerId::from("P1"), drop_act.clone())];
        let borrowed_history: Vec<_> = history.iter().collect();
        let next_state = GameLogic::apply_action(current_state.clone(), "P1", Action::Game(drop_act), None).unwrap();
        total_juggling_reward += score_rl(&current_state, &next_state, &borrowed_history, &weights).total;

        prop_assert!(total_juggling_reward < 0.0, "Item juggling (PickUp + Drop) should be net-negative, got {}", total_juggling_reward);
    }

    /// Test that taking a "productive" action is always better than passing.
    #[test]
    fn test_gradient_directionality(seed in 0u64..u64::MAX, steps in 0usize..2000) {
        let player_ids = vec!["P1".to_owned()];
        let weights = RlScoringWeights::default();
        let mut state = GameLogic::new_game(player_ids.clone(), seed);
        let mut rng = StdRng::seed_from_u64(seed);

        for _ in 0..steps {
            if state.phase == GamePhase::GameOver || state.phase == GamePhase::Victory { break; }
            let valid = get_valid_actions(&state, "P1");
            if valid.is_empty() { break; }
            let act = valid.choose(&mut rng).unwrap().clone();
            if let Ok(next) = GameLogic::apply_action(state.clone(), "P1", act, None) {
                state = next;
                sint_core::logic::resolution::resolve_proposal_queue(&mut state, false);
            }
        }

        // Ensure state is at least TacticalPlanning for testing if it ended in Lobby
        if state.phase == GamePhase::Lobby || state.phase == GamePhase::Setup || state.phase == GamePhase::MorningReport {
             state.phase = GamePhase::TacticalPlanning;
        }

        // Perturb state
        state.hull_integrity = rng.random_range(1..21);
        if state.enemy.max_hp > 0 {
            state.enemy.hp = rng.random_range(1..state.enemy.max_hp + 1);
        }
        state.phase = GamePhase::TacticalPlanning;
        state.active_situations.clear();

        if state.players.get("P1").is_none() {
            let p = Player {
                id: "P1".to_owned(),
                name: "The Reader".to_owned(),
                room_id: 1,
                hp: 3,
                ap: 10,
                inventory: Default::default(),
                status: Default::default(),
                is_ready: false,
            };
            state.players.insert(p);
        } else {
            let p = state.players.get_mut("P1").unwrap();
            p.hp = 3;
            p.ap = 10;
            p.status.clear();
            p.is_ready = false;
            p.inventory.clear();
        }

        let valid_actions = get_valid_actions(&state, "P1");
        let pass_act = ("P1".to_owned(), GameAction::Pass);
        let history = vec![pass_act];
        let borrowed_history: Vec<_> = history.iter().collect();
        let next_state_pass = GameLogic::apply_action(state.clone(), "P1", Action::Game(GameAction::Pass), None).unwrap();
        let pass_reward = score_rl(&state, &next_state_pass, &borrowed_history, &weights).total;

        for action in valid_actions {
            if let Action::Game(ga) = action {
                let mut next_state = GameLogic::apply_action(state.clone(), "P1", Action::Game(ga.clone()), None).unwrap();
                // Resolve to ensure side effects (like extinguishing) are visible in the state
                sint_core::logic::resolution::resolve_proposal_queue(&mut next_state, false);

                let step_act = ("P1".to_owned(), ga.clone());
                let history = vec![step_act];
                let borrowed_history: Vec<_> = history.iter().collect();
                let reward = score_rl(&state, &next_state, &borrowed_history, &weights).total;

                prop_assert!(reward.abs() < 200.0, "Reward magnitude too large: {} for action {:?}", reward, ga);

                if matches!(ga, GameAction::Shoot) && next_state.enemy.hp < state.enemy.hp {
                    prop_assert!(reward > pass_reward, "Hitting the boss ({}) should be better than passing ({})", reward, pass_reward);
                }

                if matches!(ga, GameAction::Extinguish) {
                    let room_id = state.players.get("P1").unwrap().room_id;
                    let parent_fire = state.map.rooms.get(&room_id).unwrap().hazards.iter().filter(|h| **h == HazardType::Fire).count();
                    let current_fire = next_state.map.rooms.get(&room_id).unwrap().hazards.iter().filter(|h| **h == HazardType::Fire).count();

                    if current_fire < parent_fire {
                        prop_assert!(reward > pass_reward, "Extinguishing fire (reward {}, fire {}->{}) should be better than passing ({})", reward, parent_fire, current_fire, pass_reward);
                    }
                }
            }
        }
    }

    #[test]
    fn test_victory_supremacy(seed in 0u64..u64::MAX) {
        let player_ids = vec!["P1".to_owned()];
        let weights = RlScoringWeights::default();
        let parent = GameLogic::new_game(player_ids.clone(), seed);

        let mut victory_state = parent.clone();
        victory_state.phase = GamePhase::Victory;

        let vic_reward = score_rl(&parent, &victory_state, &[], &weights).total;
        prop_assert!(vic_reward >= weights.victory_reward);
    }
}
