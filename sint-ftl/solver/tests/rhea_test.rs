use sint_solver::scoring::rhea::RheaScoringWeights;
use sint_solver::search::rhea::{rhea_search, RHEAConfig};
use std::time::Instant;

#[test]
fn test_rhea_smoke() {
    let config = RHEAConfig {
        players: 6,
        seed: 42,
        horizon: 5,
        generations: 5,
        population_size: 5,
        max_steps: 10,
        time_limit: 5,
        verbose: false,
    };
    let weights = RheaScoringWeights::default();
    let result = rhea_search(&config, &weights);
    assert!(result.is_some());
    let node = result.unwrap();
    assert!(!node.get_history().is_empty());
}

#[test]
fn test_rhea_determinism() {
    let config = RHEAConfig {
        players: 4,
        seed: 12345,
        horizon: 10,
        generations: 10,
        population_size: 10,
        max_steps: 20,
        time_limit: 10,
        verbose: false,
    };
    let weights = RheaScoringWeights::default();

    let result1 = rhea_search(&config, &weights).expect("RHEA failed run 1");
    let result2 = rhea_search(&config, &weights).expect("RHEA failed run 2");

    assert_eq!(
        result1.signature, result2.signature,
        "States should match for same seed"
    );
    assert_eq!(
        result1.score, result2.score,
        "Scores should match for same seed"
    );

    // Check history match
    let hist1 = result1.get_history();
    let hist2 = result2.get_history();
    assert_eq!(hist1.len(), hist2.len());
    for (i, (p1, a1)) in hist1.iter().enumerate() {
        let (p2, a2) = hist2[i];
        assert_eq!(p1, p2);
        assert_eq!(a1, a2);
    }
}

#[test]
fn test_rhea_seed_sensitivity() {
    let config1 = RHEAConfig {
        players: 4,
        seed: 100,
        horizon: 10,
        generations: 10,
        population_size: 10,
        max_steps: 20,
        time_limit: 10,
        verbose: false,
    };
    let config2 = RHEAConfig {
        seed: 200,
        ..config1
    };
    let weights = RheaScoringWeights::default();

    let result1 = rhea_search(&config1, &weights).expect("RHEA failed run 1");
    let result2 = rhea_search(&config2, &weights).expect("RHEA failed run 2");

    // It is possible they end up in same state if moves are forced, but unlikely with enough steps/generations.
    // If they are exactly the same, this test might be flaky if the game is too deterministic.
    // But with 20 steps, different seeds usually lead to different outcomes (e.g. card draws, random events).
    if result1.signature == result2.signature {
        // Check history
        let hist1 = result1.get_history();
        let hist2 = result2.get_history();
        assert_ne!(hist1, hist2, "With different seeds, history should differ");
    }
}

#[test]
fn test_rhea_time_limit() {
    let config = RHEAConfig {
        players: 4,
        seed: 999,
        horizon: 50,          // Long horizon
        generations: 1000,    // Many gens
        population_size: 100, // Large pop
        max_steps: 1000,      // Many steps
        time_limit: 1,        // Short time limit (1 sec)
        verbose: false,
    };
    let weights = RheaScoringWeights::default();

    let start = Instant::now();
    let result = rhea_search(&config, &weights);
    let duration = start.elapsed();

    assert!(result.is_some());
    // Give it a bit of slack for initialization overhead
    // We expect it to stop very close to 1s. 3s is plenty of buffer.
    assert!(duration.as_secs() < 3, "RHEA took too long: {:?}", duration);
}
