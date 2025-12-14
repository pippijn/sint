use proptest::prelude::*;
use rand::prelude::IndexedRandom;
use rand::{SeedableRng, rngs::StdRng};
use sint_core::logic::actions::get_valid_actions;
use sint_core::{GameLogic, logic::resolution, types::*};

proptest! {
    /// Test that any sequence of valid actions preserves basic game invariants.
    #[test]
    fn test_invariants_hold_after_random_actions(seed in 0u64..u64::MAX, steps in 1usize..100) {
        let player_ids = vec!["P1".to_owned(), "P2".to_owned()];
        let mut state = GameLogic::new_game(player_ids.clone(), seed);

        // Transition from Lobby -> TacticalPlanning
        for _ in 0..3 {
            for pid in &player_ids {
                state = GameLogic::apply_action(state, pid, Action::Game(GameAction::VoteReady { ready: true }), None).unwrap();
            }
        }

        prop_assert_eq!(state.phase, GamePhase::TacticalPlanning);

        let mut rng = StdRng::seed_from_u64(seed);

        for _ in 0..steps {
            if state.phase == GamePhase::GameOver || state.phase == GamePhase::Victory {
                break;
            }

            // Pick a random player who can still act
            let acting_players: Vec<_> = player_ids.iter().filter(|&&ref pid| {
                !get_valid_actions(&state, pid).is_empty()
            }).collect();

            if acting_players.is_empty() {
                // Everyone might be ready or fainted, advance phase if possible
                break;
            }

            let player_id = acting_players.choose(&mut rng).unwrap();
            let valid_actions = get_valid_actions(&state, player_id);

            // Pick a random valid action
            let action = valid_actions.choose(&mut rng).unwrap().clone();

            // Apply it
            let next_state = GameLogic::apply_action(state.clone(), player_id, action.clone(), None);

            // Property: Valid actions should never return Err
            prop_assert!(next_state.is_ok(), "Action {:?} from get_valid_actions failed for {} in phase {:?}", action, player_id, state.phase);
            state = next_state.unwrap();

            // Property: Invariants must hold
            prop_assert!(state.hull_integrity <= 20, "Hull exceeded MAX_HULL");

            for p in state.players.values() {
                prop_assert!(p.hp <= 3, "Player HP exceeded MAX_PLAYER_HP");
                prop_assert!(p.ap >= 0, "Player AP dropped below zero");

                // Fainted Status Invariant
                if p.hp <= 0 {
                    prop_assert!(p.status.contains(&PlayerStatus::Fainted), "Player {} has {} HP but is not Fainted", p.id, p.hp);
                } else {
                    prop_assert!(!p.status.contains(&PlayerStatus::Fainted), "Player {} has {} HP but is Fainted", p.id, p.hp);
                }

                // Inventory Limit Invariants
                let nut_count = p.peppernut_count();
                let special_count = p.special_item_count();
                let has_wheelbarrow = p.has_wheelbarrow();

                if has_wheelbarrow {
                    prop_assert!(nut_count <= 5, "Player {} exceeded Wheelbarrow capacity ({} > 5)", p.id, nut_count);
                    prop_assert_eq!(special_count, 1, "Player {} with Wheelbarrow should only have 1 special item", p.id);
                } else {
                    prop_assert!(nut_count + special_count <= 2, "Player {} exceeded hand capacity ({} + {} > 2)", p.id, nut_count, special_count);
                }
            }

            // Disabled System Invariants (Check against PROJECTED state)
            {
                let mut projected_state = state.clone();
                resolution::resolve_proposal_queue(&mut projected_state, true);

                for room in projected_state.map.rooms.values() {
                    let is_disabled = !room.hazards.is_empty();
                    if is_disabled && room.system.is_some() {
                        for pid in &player_ids {
                             let actions = get_valid_actions(&state, pid);
                             for action in actions {
                                if let Action::Game(game_action) = action {
                                    match game_action {
                                        GameAction::Bake | GameAction::Shoot | GameAction::RaiseShields | GameAction::EvasiveManeuvers | GameAction::Lookout | GameAction::FirstAid { .. } => {
                                            // If the player is in THIS disabled room in the PROJECTION, they shouldn't see these actions
                                            if let Some(p) = projected_state.players.get(pid) {
                                                if p.room_id == room.id {
                                                     prop_assert!(false, "System action {:?} available for {} in projected disabled room {}", game_action, pid, room.id);
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                             }
                        }
                    }
                }
            }
        }
    }

    /// Test that serializing and deserializing a state is an identity function.
    #[test]
    fn test_serialization_roundtrip(seed in 0u64..u64::MAX) {
        let state = GameLogic::new_game(vec!["P1".to_owned()], seed);
        let json = serde_json::to_string(&state).unwrap();
        let state_back: GameState = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(state, state_back);
    }

    /// Test that undoing a queued action restores the state (functionally).
    #[test]
    fn test_undo_is_inverse(seed in 0u64..u64::MAX) {
        let mut state = GameLogic::new_game(vec!["P1".to_owned()], seed);

        // Setup state to TacticalPlanning
        state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::VoteReady { ready: true }), None).unwrap();
        state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::VoteReady { ready: true }), None).unwrap();
        state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::VoteReady { ready: true }), None).unwrap();

        let mut rng = StdRng::seed_from_u64(seed);

        let valid_actions = get_valid_actions(&state, "P1");
        // Filter for actions that can be undone (queued actions)
        let undoable_actions: Vec<_> = valid_actions.into_iter().filter(|a| {
            if let Action::Game(ga) = a {
                match ga {
                    GameAction::Move { .. } | GameAction::Bake | GameAction::Shoot |
                    GameAction::RaiseShields | GameAction::EvasiveManeuvers |
                    GameAction::Extinguish | GameAction::Repair | GameAction::PickUp { .. } |
                    GameAction::Lookout | GameAction::FirstAid { .. } | GameAction::Interact |
                    GameAction::Throw { .. } | GameAction::Revive { .. } => true,
                    _ => false
                }
            } else {
                false
            }
        }).collect();

        if !undoable_actions.is_empty() {
            let action = undoable_actions.choose(&mut rng).unwrap().clone();

            let original_state = state.clone();
            let state_after_action = GameLogic::apply_action(state.clone(), "P1", action.clone(), None).unwrap();

            // Get the action ID from the queue
            let action_id = state_after_action.proposal_queue.last().unwrap().id;

            let state_after_undo = GameLogic::apply_action(state_after_action, "P1", Action::Game(GameAction::Undo { action_id }), None).unwrap();

            // Check equivalence (excluding sequence_id and rng_seed which advance)
            prop_assert_eq!(&state_after_undo.players, &original_state.players);
            prop_assert_eq!(&state_after_undo.map, &original_state.map);
            prop_assert_eq!(state_after_undo.hull_integrity, original_state.hull_integrity);
            prop_assert_eq!(&state_after_undo.proposal_queue, &original_state.proposal_queue);
            prop_assert_eq!(state_after_undo.phase, original_state.phase);
        }
    }

    /// Test that applying the same action to the same state is deterministic.
    #[test]
    fn test_deterministic_execution(seed in 0u64..u64::MAX) {
        let state = GameLogic::new_game(vec!["P1".to_owned()], seed);
        let mut rng = StdRng::seed_from_u64(seed);

        let valid_actions = get_valid_actions(&state, "P1");
        if !valid_actions.is_empty() {
            let action = valid_actions.choose(&mut rng).unwrap().clone();

            let state1 = GameLogic::apply_action(state.clone(), "P1", action.clone(), None).unwrap();
            let state2 = GameLogic::apply_action(state.clone(), "P1", action.clone(), None).unwrap();

            prop_assert_eq!(state1, state2);
        }
    }

    /// Test that the generated map is always fully connected.
    #[test]
    fn test_map_connectivity(layout in prop_oneof![Just(MapLayout::Star), Just(MapLayout::Torus)]) {
        let map = sint_core::logic::map_gen::generate_map(layout);
        let room_ids: Vec<RoomId> = map.rooms.keys().collect();
        if room_ids.is_empty() { return Ok(()); }

        let start_room = room_ids[0];
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();

        queue.push_back(start_room);
        visited.insert(start_room);

        while let Some(curr) = queue.pop_front() {
            if let Some(room) = map.rooms.get(&curr) {
                for &neighbor in &room.neighbors {
                    // Symmetry check: if neighbor has curr as neighbor
                    let neighbor_room = map.rooms.get(&neighbor).expect("Neighbor must exist");
                    prop_assert!(neighbor_room.neighbors.contains(&curr), "Map neighbor relationship is not symmetric: {} -> {}, but not {} -> {}", curr, neighbor, neighbor, curr);

                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        prop_assert_eq!(visited.len(), room_ids.len(), "Map is not fully connected for layout {:?}", layout);
    }

    /// Test that non-free actions always deplete AP.
    #[test]
    fn test_ap_depletion(seed in 0u64..u64::MAX) {
        let mut state = GameLogic::new_game(vec!["P1".to_owned()], seed);

        // Setup state to TacticalPlanning
        state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::VoteReady { ready: true }), None).unwrap();
        state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::VoteReady { ready: true }), None).unwrap();
        state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::VoteReady { ready: true }), None).unwrap();

        let mut rng = StdRng::seed_from_u64(seed);

        let valid_actions = get_valid_actions(&state, "P1");
        let costly_actions: Vec<_> = valid_actions.into_iter().filter(|a| {
            if let Action::Game(ga) = a {
                // Check if the action actually costs AP in the current state
                sint_core::logic::actions::action_cost(&state, "P1", ga) > 0
            } else {
                false
            }
        }).collect();

        if !costly_actions.is_empty() {
            let action = costly_actions.choose(&mut rng).unwrap().clone();
            let p_before = state.players.get("P1").unwrap().clone();
            let ap_before = p_before.ap;
            let room_before = p_before.room_id;

            let next_state = GameLogic::apply_action(state, "P1", action.clone(), None).unwrap();
            let p_after = next_state.players.get("P1").unwrap();
            let ap_after = p_after.ap;

            prop_assert!(ap_after < ap_before, "Costly action {:?} did not deplete AP ({} -> {}) at room {}", action, ap_before, ap_after, room_before);
        }
    }

    /// Test that entering a Rest Round resets AP to 6.
    #[test]
    fn test_rest_round_ap_reset(seed in 0u64..u64::MAX) {
        let mut state = GameLogic::new_game(vec!["P1".to_owned()], seed);

        // Setup state to Execution phase (simplest path to EnemyAction)
        state.phase = GamePhase::Execution;
        state.enemy.state = EnemyState::Defeated;
        // MUST have 0 AP to transition to EnemyAction
        state.players.get_mut("P1").unwrap().ap = 0;

        // Trigger transition to EnemyAction
        let state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::VoteReady { ready: true }), None).unwrap();
        prop_assert_eq!(state.phase, GamePhase::EnemyAction);

        // Trigger transition to MorningReport (Rest Round starts)
        let state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::VoteReady { ready: true }), None).unwrap();
        prop_assert_eq!(state.phase, GamePhase::MorningReport);

        prop_assert!(state.is_resting, "Game should be in Rest Round after defeating boss and advancing from EnemyAction");
        for p in state.players.values() {
            prop_assert_eq!(p.ap, 6, "Player {} should have 6 AP in Rest Round", p.id);
        }
    }

    /// Test that Cargo Repair respects MAX_HULL.
    #[test]
    fn test_cargo_repair_limit(seed in 0u64..u64::MAX) {
        let mut state = GameLogic::new_game(vec!["P1".to_owned()], seed);
        let cargo_id = sint_core::logic::find_room_with_system_in_map(&state.map, SystemType::Cargo).unwrap();

        // Put player in Cargo
        state.players.get_mut("P1").unwrap().room_id = cargo_id;
        state.hull_integrity = 19;
        state.phase = GamePhase::TacticalPlanning;

        // Repair 1: Should work
        let state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::Repair), None).unwrap();
        // Note: Repair is queued in TacticalPlanning, not executed immediately.
        // We need to resolve it.
        let mut state = state;
        sint_core::logic::resolution::resolve_proposal_queue(&mut state, false);
        prop_assert_eq!(state.hull_integrity, 20);

        // Repair 2: Should not be valid now
        let valid_actions = get_valid_actions(&state, "P1");
        let can_repair = valid_actions.iter().any(|a| matches!(a, Action::Game(GameAction::Repair)));
        prop_assert!(!can_repair, "Should not be able to repair when Hull is at MAX");
    }

    /// Test that Water destroys Peppernuts at the end of the round (during transition to EnemyAction).
    #[test]
    fn test_water_destroys_nuts_at_end_of_round(seed in 0u64..u64::MAX) {
        let mut state = GameLogic::new_game(vec!["P1".to_owned()], seed);
        let kitchen_id = sint_core::logic::find_room_with_system_in_map(&state.map, SystemType::Kitchen).unwrap();

        // Setup: Room with Water and Peppernuts
        let room = state.map.rooms.get_mut(&kitchen_id).unwrap();
        room.add_hazard(HazardType::Water);
        room.add_item(ItemType::Peppernut);

        // Verify they coexist during TacticalPlanning
        state.phase = GamePhase::TacticalPlanning;
        prop_assert!(state.map.rooms.get(&kitchen_id).unwrap().items.contains(&ItemType::Peppernut));

        // Advance to Execution
        state.players.get_mut("P1").unwrap().is_ready = true;
        let mut state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::VoteReady { ready: true }), None).unwrap();
        prop_assert_eq!(state.phase, GamePhase::Execution);

        // In Execution, they still coexist
        prop_assert!(state.map.rooms.get(&kitchen_id).unwrap().items.contains(&ItemType::Peppernut));

        // Advance to EnemyAction (resolves hazards)
        state.players.get_mut("P1").unwrap().ap = 0;
        let state = GameLogic::apply_action(state, "P1", Action::Game(GameAction::VoteReady { ready: true }), None).unwrap();
        prop_assert_eq!(state.phase, GamePhase::EnemyAction);

        // Now Peppernuts should be gone
        let has_nuts = state.map.rooms.get(&kitchen_id).unwrap().items.contains(&ItemType::Peppernut);
        prop_assert!(!has_nuts, "Peppernuts should have been destroyed by Water during transition to EnemyAction");
    }
}
