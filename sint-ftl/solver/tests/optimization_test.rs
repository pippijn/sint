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
    let results = evaluate_batch(&config, &genomes, &tx, 0, &existing);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].score, 99.0);
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
