use sint_core::logic::GameLogic;
use sint_core::types::{GameAction, GamePhase, PlayerId};
use sint_solver::scoring::rl::{RlScoringWeights, score_rl};

#[test]
fn test_rl_score_bounds() {
    let player_ids = vec![
        "P1".to_string(),
        "P2".to_string(),
        "P3".to_string(),
        "P4".to_string(),
    ];
    let weights = RlScoringWeights::default();

    for seed in 0..100 {
        let state = GameLogic::new_game(player_ids.clone(), seed as u64);
        let parent = state.clone();

        let history: Vec<(PlayerId, GameAction)> = vec![];
        let borrowed_history: Vec<&(PlayerId, GameAction)> = history.iter().collect();

        let details = score_rl(&parent, &state, &borrowed_history, &weights);

        // Single step delta should not be astronomical
        assert!(
            details.total.abs() < 1000.0,
            "Score {} is too large for initial state (seed {})",
            details.total,
            seed
        );
    }
}

#[test]
fn test_stalling_is_negative() {
    let player_ids = vec![
        "P1".to_string(),
        "P2".to_string(),
        "P3".to_string(),
        "P4".to_string(),
    ];
    let weights = RlScoringWeights::default();

    let parent = GameLogic::new_game(player_ids.clone(), 12345);
    let mut current = parent.clone();

    // Simulate a "Pass" action and a turn advancement
    current.turn_count += 1;

    let history: Vec<(PlayerId, GameAction)> = vec![(PlayerId::from("P1"), GameAction::Pass)];
    let borrowed_history: Vec<&(PlayerId, GameAction)> = history.iter().collect();

    let details = score_rl(&parent, &current, &borrowed_history, &weights);

    // Total reward should be negative to prevent infinite survival stalling
    // Survival bonus at max hull is (20/20) * 0.1 = 0.1
    // Penalties are weights.turn_penalty (2.0) + weights.step_penalty (0.2) = 2.2
    // Net should be roughly -2.1
    assert!(
        details.total < 0.0,
        "Stalling (Pass) should result in a negative reward, got {}",
        details.total
    );
    assert!(
        details.total > -10.0,
        "Stalling penalty is too extreme: {}",
        details.total
    );
}

#[test]
fn test_victory_reward_magnitude() {
    let player_ids = vec![
        "P1".to_string(),
        "P2".to_string(),
        "P3".to_string(),
        "P4".to_string(),
    ];
    let weights = RlScoringWeights::default();

    let parent = GameLogic::new_game(player_ids.clone(), 12345);
    let mut current = parent.clone();
    current.phase = GamePhase::Victory;

    let history: Vec<(PlayerId, GameAction)> = vec![];
    let borrowed_history: Vec<&(PlayerId, GameAction)> = history.iter().collect();

    let details = score_rl(&parent, &current, &borrowed_history, &weights);

    assert_eq!(
        details.total, weights.victory_reward,
        "Victory reward should exactly match weights.victory_reward"
    );
}

#[test]
fn test_defeat_reward_magnitude() {
    let player_ids = vec![
        "P1".to_string(),
        "P2".to_string(),
        "P3".to_string(),
        "P4".to_string(),
    ];
    let weights = RlScoringWeights::default();

    let parent = GameLogic::new_game(player_ids.clone(), 12345);
    let mut current = parent.clone();
    current.phase = GamePhase::GameOver;

    let history: Vec<(PlayerId, GameAction)> = vec![];
    let borrowed_history: Vec<&(PlayerId, GameAction)> = history.iter().collect();

    let details = score_rl(&parent, &current, &borrowed_history, &weights);

    assert_eq!(
        details.total, weights.defeat_penalty,
        "Defeat penalty should exactly match weights.defeat_penalty"
    );
}

#[test]
fn test_no_survival_stalling_at_low_hp() {
    let player_ids = vec![
        "P1".to_string(),
        "P2".to_string(),
        "P3".to_string(),
        "P4".to_string(),
    ];
    let weights = RlScoringWeights::default();

    let mut parent = GameLogic::new_game(player_ids.clone(), 12345);
    parent.hull_integrity = 1; // Critical hull

    let mut current = parent.clone();
    current.turn_count += 1;

    let history: Vec<(PlayerId, GameAction)> = vec![(PlayerId::from("P1"), GameAction::Pass)];
    let borrowed_history: Vec<&(PlayerId, GameAction)> = history.iter().collect();

    let details = score_rl(&parent, &current, &borrowed_history, &weights);

    // Survival bonus is (1/20) * 0.1 = 0.005
    // Penalties are 2.2
    assert!(
        details.total < -2.0,
        "Stalling at 1 HP should be heavily penalized, got {}",
        details.total
    );
}
