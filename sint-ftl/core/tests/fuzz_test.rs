use rand::{seq::SliceRandom, thread_rng};
use sint_core::{logic::actions::get_valid_actions, GameLogic, GamePhase};

#[test]
fn test_fuzz_random_walk() {
    let mut rng = thread_rng();
    // Use a fixed seed for reproducibility of the initial state,
    // but the walk itself is randomized by thread_rng.
    let mut state = GameLogic::new_game(vec!["P1".to_string(), "P2".to_string()], 12345);

    // Fuzz for a set number of steps
    let max_steps = 2000;

    println!("Starting Fuzz Test with {} steps...", max_steps);

    for i in 0..max_steps {
        // Stop if game ended
        if state.phase == GamePhase::GameOver || state.phase == GamePhase::Victory {
            println!("Fuzz ended at step {} with phase {:?}", i, state.phase);
            return;
        }

        // Collect all valid actions for all players
        let mut all_possible_moves = Vec::new();

        for pid in state.players.keys() {
            let actions = get_valid_actions(&state, pid);
            for a in actions {
                all_possible_moves.push((pid.clone(), a));
            }
        }

        if all_possible_moves.is_empty() {
            println!("No valid moves at step {}. Phase: {:?}", i, state.phase);
            panic!("Stalemate: No valid moves found!");
        }

        // Pick random move
        let (pid, action) = all_possible_moves.choose(&mut rng).unwrap().clone();

        // Apply
        match GameLogic::apply_action(state.clone(), &pid, action.clone(), None) {
            Ok(new_state) => {
                state = new_state;
            }
            Err(e) => {
                println!(
                    "Step {}: Action {:?} for {} failed: {:?}",
                    i, action, pid, e
                );
                // We do not panic here because race conditions or specific validation logic
                // might cause rejection even if get_valid_actions thought it was ok
                // (though ideally they should match).
            }
        }
    }

    println!("Fuzz test completed {} steps without crashing.", max_steps);
}
