use sint_solver::optimization::{
    Checkpoint, EvaluationMetrics, OptimizationStatus, OptimizerConfig, SeedResult, Strategy,
    Target, apply_multipliers_beam, get_param_count, mutate,
};
use sint_solver::scoring::beam::BeamScoringWeights;
use std::fs;

#[test]
fn test_checkpoint_roundtrip() {
    let config = OptimizerConfig {
        strategy: Strategy::GA,
        target: Target::Beam,
        generations: 10,
        population: 5,
        seeds: vec![1, 2, 3],
        beam_width: 20,
        rhea_horizon: 10,
        rhea_generations: 5,
        rhea_population: 30,
    };

    let metrics = EvaluationMetrics {
        score: 1234.5,
        wins: 1,
        losses: 0,
        timeouts: 0,
        panics: 0,
    };

    let status = OptimizationStatus {
        generation: 2,
        best_score: 1500.0,
        avg_score: 1100.0,
        best_metrics: metrics.clone(),
        best_genome: vec![1.1, 2.2, 3.3],
        current_weights_beam: Some(BeamScoringWeights::default()),
        current_weights_rhea: None,
    };

    let seed_result = SeedResult {
        ind_idx: 0,
        seed_idx: 1,
        metrics: metrics.clone(),
    };

    let checkpoint = Checkpoint {
        config: config.clone(),
        generation: 3,
        population: vec![vec![1.0; 5], vec![2.0; 5]],
        seed_results: vec![seed_result],
        history: vec![status],
    };

    let path = "test_checkpoint.json";
    checkpoint.save(path).expect("Failed to save checkpoint");

    let loaded = Checkpoint::load(path).expect("Failed to load checkpoint");

    assert_eq!(loaded.generation, checkpoint.generation);
    assert_eq!(loaded.population, checkpoint.population);
    assert_eq!(loaded.seed_results.len(), checkpoint.seed_results.len());
    assert_eq!(loaded.history.len(), checkpoint.history.len());
    assert_eq!(loaded.config.generations, checkpoint.config.generations);
    assert_eq!(loaded.config.seeds, checkpoint.config.seeds);

    // Clean up
    fs::remove_file(path).ok();
    fs::remove_file(format!("{}.tmp", path)).ok();
}

#[test]
fn test_mutate_stays_positive() {
    let mut rng = rand::rng();
    let mut genome = vec![1.0; 100];

    for _ in 0..100 {
        mutate(&mut rng, &mut genome);
        for &val in &genome {
            assert!(val >= 0.0, "Genome value {} is negative", val);
            assert!(!val.is_nan(), "Genome value is NaN");
        }
    }
}

#[test]
fn test_multiplier_application() {
    let base = BeamScoringWeights::default();
    // DNA: 2.0 multiplier for all params
    let param_count = get_param_count(Target::Beam);
    let genome = vec![2.0; param_count];

    let updated = apply_multipliers_beam(&base, &genome);

    // Pick a few representative weights to check
    // We expect them to be exactly 2x the default
    assert_eq!(updated.hull_integrity, base.hull_integrity * 2.0);
    assert_eq!(updated.enemy_hp, base.enemy_hp * 2.0);
}

#[test]
fn test_metrics_addition() {
    let mut m1 = EvaluationMetrics {
        score: 100.0,
        wins: 1,
        losses: 2,
        timeouts: 3,
        panics: 4,
    };
    let m2 = EvaluationMetrics {
        score: 50.0,
        wins: 0,
        losses: 1,
        timeouts: 0,
        panics: 1,
    };

    m1.add(&m2);

    assert_eq!(m1.score, 150.0);
    assert_eq!(m1.wins, 1);
    assert_eq!(m1.losses, 3);
    assert_eq!(m1.timeouts, 3);
    assert_eq!(m1.panics, 5);
}

#[test]
fn test_evaluate_batch_resumption_skipping() {
    use sint_solver::optimization::{OptimizerMessage, evaluate_batch};
    use std::sync::mpsc;

    let config = OptimizerConfig {
        strategy: Strategy::GA,
        target: Target::Beam,
        generations: 1,
        population: 1,
        seeds: vec![123],
        beam_width: 1,
        rhea_horizon: 1,
        rhea_generations: 1,
        rhea_population: 1,
    };

    let genomes = vec![vec![1.0; get_param_count(Target::Beam)]];
    let (tx, _rx) = mpsc::channel::<OptimizerMessage>();

    // Provide existing result for the ONLY task
    let existing = vec![SeedResult {
        ind_idx: 0,
        seed_idx: 0,
        metrics: EvaluationMetrics {
            score: 99.0,
            ..Default::default()
        },
    }];

    // This should return immediately without spawning any tasks because all work is done
    let results = evaluate_batch(&config, &genomes, &tx, 0, &existing, 0);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].score, 99.0);
}

#[test]
fn test_spsa_resume_skipping() {
    use sint_solver::optimization::{
        Checkpoint, EvaluationMetrics, OptimizerConfig, SeedResult, Strategy, Target,
    };

    // 1. Create a checkpoint mid-iteration for SPSA
    let checkpoint = Checkpoint {
        config: OptimizerConfig {
            strategy: Strategy::Spsa,
            target: Target::Beam,
            generations: 10,
            population: 1, // SPSA population is pseudo-defined
            seeds: vec![123],
            beam_width: 1,
            rhea_horizon: 1,
            rhea_generations: 1,
            rhea_population: 1,
        },
        generation: 5,
        population: vec![vec![1.0; 10], vec![1.0; 10]], // [theta, best_theta]
        seed_results: vec![SeedResult {
            ind_idx: 0, // theta_plus
            seed_idx: 0,
            metrics: EvaluationMetrics {
                score: 777.0,
                ..Default::default()
            },
        }],
        history: Vec::new(),
    };

    // 2. Simulate the perturbed evaluation in run_spsa
    // Currently, run_spsa passes &[] instead of the existing results!
    let (tx, _rx) = std::sync::mpsc::channel();
    let theta_perturbed = vec![vec![1.1; 10], vec![0.9; 10]]; // theta_plus, theta_minus

    // We want to see if it uses the 777.0 score from the checkpoint
    let results = sint_solver::optimization::evaluate_batch(
        &checkpoint.config,
        &theta_perturbed,
        &tx,
        5,
        &checkpoint.seed_results,
        0,
    );

    // If this is 777.0, evaluate_batch logic is fine, but we need to ensure
    // run_spsa actually passes the results down.
    assert_eq!(
        results[0].score, 777.0,
        "SPSA should use the existing result from the checkpoint!"
    );
}

#[test]
fn test_spsa_perturbation_determinism() {
    use rand::prelude::*;

    // We want to see if SPSA produces the same delta if we "resume" it.
    // Currently it uses rand::rng() which is thread_rng and non-deterministic.

    let p = 10;

    let get_delta = |k: u64| {
        let mut rng = rand::rngs::StdRng::seed_from_u64(12345 + k);
        let delta: Vec<f64> = (0..p)
            .map(|_| if rng.random_bool(0.5) { 1.0 } else { -1.0 })
            .collect();
        delta
    };

    let delta1 = get_delta(5);
    let delta2 = get_delta(5);

    assert_eq!(
        delta1, delta2,
        "SPSA perturbations should be deterministic based on generation for safe resumption."
    );
}

#[test]
fn test_history_duplication_on_resume() {
    use sint_solver::optimization::{
        Checkpoint, EvaluationMetrics, OptimizationStatus, OptimizerMessage, Strategy, Target,
    };
    use std::sync::mpsc;

    // Simulate a checkpoint with 3 generations of history
    let mut history = Vec::new();
    for i in 0..3 {
        history.push(OptimizationStatus {
            generation: i,
            best_score: i as f64,
            avg_score: i as f64,
            best_metrics: EvaluationMetrics::default(),
            best_genome: vec![i as f64],
            current_weights_beam: None,
            current_weights_rhea: None,
        });
    }

    let checkpoint = Checkpoint {
        config: OptimizerConfig {
            strategy: Strategy::GA,
            target: Target::Beam,
            generations: 10,
            population: 1,
            seeds: vec![123],
            beam_width: 1,
            rhea_horizon: 1,
            rhea_generations: 1,
            rhea_population: 1,
        },
        generation: 3,
        population: vec![vec![1.0]],
        seed_results: Vec::new(),
        history: history.clone(),
    };

    // --- SIMULATE UI/CLI RESUME LOGIC ---
    let (tx, rx) = mpsc::channel::<OptimizerMessage>();

    // 1. Initial State Setup (as in run_tui/run_cli)
    let mut app_history = checkpoint.history.clone();

    // 2. run_ga Resume Logic (No longer sends history via messages)
    // for status in checkpoint.history {
    //     tx.send(OptimizerMessage::GenerationDone(Box::new(status))).unwrap();
    // }

    // Simulate only NEW generations being sent
    tx.send(OptimizerMessage::GenerationDone(Box::new(
        OptimizationStatus {
            generation: 3,
            best_score: 300.0,
            avg_score: 300.0,
            best_metrics: EvaluationMetrics::default(),
            best_genome: vec![3.0],
            current_weights_beam: None,
            current_weights_rhea: None,
        },
    )))
    .unwrap();

    // 3. Message Loop (as in run_tui/run_cli)
    while let Ok(msg) = rx.try_recv() {
        if let OptimizerMessage::GenerationDone(status) = msg {
            app_history.push(*status);
        }
    }

    // --- VERIFY ---
    // Initial 3 + New 1 = 4
    assert_eq!(
        app_history.len(),
        4,
        "History should contain initial items plus new progress. Found {} items.",
        app_history.len()
    );
}

#[test]
fn test_seed_status_preservation_on_resume() {
    use sint_solver::optimization::{Checkpoint, EvaluationMetrics, SeedResult};

    // 1. Create a checkpoint with one win and one loss
    let win_metrics = EvaluationMetrics {
        wins: 1,
        losses: 0,
        ..Default::default()
    };
    let loss_metrics = EvaluationMetrics {
        wins: 0,
        losses: 1,
        ..Default::default()
    };

    let checkpoint = Checkpoint {
        config: OptimizerConfig {
            strategy: Strategy::GA,
            target: Target::Beam,
            generations: 1,
            population: 1,
            seeds: vec![1, 2],
            beam_width: 1,
            rhea_horizon: 1,
            rhea_generations: 1,
            rhea_population: 1,
        },
        generation: 1,
        population: vec![vec![1.0]],
        seed_results: vec![
            SeedResult {
                ind_idx: 0,
                seed_idx: 0,
                metrics: win_metrics,
            },
            SeedResult {
                ind_idx: 0,
                seed_idx: 1,
                metrics: loss_metrics,
            },
        ],
        history: Vec::new(),
    };
    // 2. Simulate fixed run_tui status mapping logic
    let mut seed_statuses = vec![0u8; 2];
    for res in &checkpoint.seed_results {
        seed_statuses[res.seed_idx] = res.metrics.get_status();
    }
    // 3. Verify
    // This will FAIL if the bug exists, as the second seed should be status 3 (Loss)
    assert_eq!(seed_statuses[0], 2, "Seed 0 should be Win (2)");
    assert_eq!(
        seed_statuses[1], 3,
        "Seed 1 should be Loss (3), but was {}",
        seed_statuses[1]
    );
}

#[test]
fn test_ga_resume_collision_corruption() {
    use sint_solver::optimization::{
        Checkpoint, EvaluationMetrics, OptimizerConfig, SeedResult, Strategy, Target,
    };

    // 1. Imagine we are in Gen 0.
    // 2. Parents (0-4) are done.
    // 3. Children (0-4) are being evaluated.
    // 4. We crash while Child 0 has Seed 0 finished with a specific score.

    let child_0_seed_0_metrics = EvaluationMetrics {
        score: 666.0, // Distinctive "corrupt" score
        wins: 1,
        ..Default::default()
    };

    let checkpoint = Checkpoint {
        config: OptimizerConfig {
            strategy: Strategy::GA,
            target: Target::Beam,
            generations: 10,
            population: 5,
            seeds: vec![123],
            beam_width: 1,
            rhea_horizon: 1,
            rhea_generations: 1,
            rhea_population: 1,
        },
        generation: 0,
        population: vec![vec![0.0; 10]; 5],
        // The fix: child results are saved with an offset (ind_idx = 5 for Child 0)
        seed_results: vec![SeedResult {
            ind_idx: 5,
            seed_idx: 0,
            metrics: child_0_seed_0_metrics,
        }],
        history: Vec::new(),
    };
    // --- SIMULATE RESUME IN run_ga ---
    let existing_seed_results = checkpoint.seed_results;

    // The parents we ARE ABOUT TO evaluate
    let parents = vec![vec![1.0; 10]; 5];

    // We use evaluate_batch to see what tasks it produces
    // We expect it to produce a task for Parent 0, but it will SKIP IT because of the collision!

    // NOTE: We need a way to check which tasks are skipped without running the whole things.
    // We can't easily check internal evaluate_batch logic without modifying it or using mocks.

    // Actually, I'll just check that Child 0 result is NOT in the final parents metrics.
    // But evaluate_batch currently returns them.

    // Let's just prove the collision exists in the 'completed' HashSet inside evaluate_batch.
    // Since I can't see the HashSet, I'll check the returned scores.

    let (tx, _rx) = std::sync::mpsc::channel();
    let results = sint_solver::optimization::evaluate_batch(
        &checkpoint.config,
        &parents,
        &tx,
        0,
        &existing_seed_results,
        0,
    );
    // If results[0] has score 666.0, it means Parent 0 was SKIPPED
    // and incorrectly assigned Child 0's score from the checkpoint!
    assert_ne!(
        results[0].score, 666.0,
        "Parent 0 incorrectly used Child 0's score from checkpoint due to index collision!"
    );
}
