use sint_solver::scoring::beam::BeamScoringWeights;
use sint_solver::search::SearchProgress;
use sint_solver::search::beam::{BeamSearchConfig, beam_search};
use std::time::Instant;

#[test]
fn test_beam_smoke() {
    let config = BeamSearchConfig {
        players: 6,
        seed: 42,
        width: 10,
        steps: 50, // Increased to ensure round transition
        time_limit: 5,
        verbose: false,
    };
    let weights = BeamScoringWeights::default();
    let result = beam_search(&config, &weights, None::<fn(SearchProgress)>);
    assert!(result.is_some());
}

#[test]
fn test_beam_determinism() {
    let config = BeamSearchConfig {
        players: 4,
        seed: 12345,
        width: 20,
        steps: 20,
        time_limit: 10,
        verbose: false,
    };
    let weights = BeamScoringWeights::default();

    let result1 =
        beam_search(&config, &weights, None::<fn(SearchProgress)>).expect("Beam failed run 1");
    let result2 =
        beam_search(&config, &weights, None::<fn(SearchProgress)>).expect("Beam failed run 2");

    assert_eq!(
        result1.signature, result2.signature,
        "States should match for same seed"
    );
    assert_eq!(
        result1.score, result2.score,
        "Scores should match for same seed"
    );

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
fn test_beam_time_limit() {
    let config = BeamSearchConfig {
        players: 4,
        seed: 999,
        width: 500,    // Large width
        steps: 5000,   // Many steps
        time_limit: 1, // Short time limit
        verbose: false,
    };
    let weights = BeamScoringWeights::default();

    let start = Instant::now();
    let result = beam_search(&config, &weights, None::<fn(SearchProgress)>);
    let duration = start.elapsed();

    assert!(result.is_some());
    // Give a buffer for init and overhead
    assert!(
        duration.as_secs() < 3,
        "Beam search took too long: {:?}",
        duration
    );
}

#[test]
fn test_beam_width_effect() {
    let weights = BeamScoringWeights::default();
    let seed = 12345;

    let config_narrow = BeamSearchConfig {
        players: 4,
        seed,
        width: 1, // Greedy
        steps: 20,
        time_limit: 5,
        verbose: false,
    };

    let config_wide = BeamSearchConfig {
        players: 4,
        seed,
        width: 50,
        steps: 20,
        time_limit: 5,
        verbose: false,
    };

    let res_narrow =
        beam_search(&config_narrow, &weights, None::<fn(SearchProgress)>).expect("Narrow failed");
    let res_wide =
        beam_search(&config_wide, &weights, None::<fn(SearchProgress)>).expect("Wide failed");

    // Wide beam should generally produce equal or better score
    assert!(res_wide.score >= res_narrow.score - 0.001);
}
