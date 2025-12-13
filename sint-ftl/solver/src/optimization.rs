use crate::scoring::beam::BeamScoringWeights;
use crate::scoring::rhea::RheaScoringWeights;
use crate::search::beam::{beam_search, BeamSearchConfig, SearchProgress};
use crate::search::rhea::{rhea_search, RHEAConfig};
use rand::prelude::*;
use rayon::prelude::*;
use sint_core::types::GamePhase;
use std::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub enum OptimizerMessage {
    GenerationDone(OptimizationStatus),
    IndividualDone {
        generation: usize,
        index: usize,
        score: f64,
        metrics: EvaluationMetrics,
    },
    IndividualUpdate {
        generation: usize,
        index: usize,
        progress: SearchProgress,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Strategy {
    GA,
    SPSA,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Target {
    Beam,
    Rhea,
}

#[derive(Clone, Debug)]
pub struct OptimizerConfig {
    pub strategy: Strategy,
    pub target: Target,
    pub generations: usize,
    pub population: usize,
    pub seeds: Vec<u64>,
    pub rhea_horizon: usize,
    pub rhea_generations: usize,
    pub rhea_population: usize,
}

#[derive(Clone, Debug, Default)]
pub struct EvaluationMetrics {
    pub score: f64,
    pub wins: usize,
    pub losses: usize,
    pub timeouts: usize,
    pub panics: usize, // None returned
}

impl EvaluationMetrics {
    fn add(&mut self, other: &EvaluationMetrics) {
        self.score += other.score;
        self.wins += other.wins;
        self.losses += other.losses;
        self.timeouts += other.timeouts;
        self.panics += other.panics;
    }

    fn average(&mut self, count: usize) {
        if count > 0 {
            self.score /= count as f64;
            // Counts are totals across seeds, so we keep them as totals or average them?
            // Usually we want totals across seeds for a single genome.
            // But if aggregating across population, maybe average?
            // Let's keep them as totals for now, assuming this struct represents ONE genome's performance across N seeds.
        }
    }
}

#[derive(Clone, Debug)]
pub struct OptimizationStatus {
    pub generation: usize,
    pub best_score: f64,
    pub avg_score: f64,
    pub best_metrics: EvaluationMetrics, // Metrics of the best individual
    pub best_genome: Vec<f64>,
    pub current_weights_beam: Option<BeamScoringWeights>,
    pub current_weights_rhea: Option<RheaScoringWeights>,
}

fn get_base_weights_beam() -> BeamScoringWeights {
    BeamScoringWeights::default()
}

fn get_base_weights_rhea() -> RheaScoringWeights {
    RheaScoringWeights::default()
}

// Map multipliers vector to BeamScoringWeights
pub fn apply_multipliers_beam(base: &BeamScoringWeights, m: &[f64]) -> BeamScoringWeights {
    let mut w = base.clone();
    let mut i = 0;

    // Ensure we don't go out of bounds if param count mismatches, though we should sync them.
    if m.len() < 59 {
        panic!(
            "Multiplier vector too short for Beam weights. Expected 59, got {}",
            m.len()
        );
    }

    w.hull_integrity *= m[i];
    i += 1;
    w.hull_delta_penalty *= m[i];
    i += 1;
    w.enemy_hp *= m[i];
    i += 1;
    w.player_hp *= m[i];
    i += 1;
    w.ap_balance *= m[i];
    i += 1;
    w.fire_penalty_base *= m[i];
    i += 1;
    w.water_penalty *= m[i];
    i += 1;
    w.active_situation_penalty *= m[i];
    i += 1;
    w.threat_player_penalty *= m[i];
    i += 1;
    w.threat_system_penalty *= m[i];
    i += 1;
    w.death_penalty *= m[i];
    i += 1;
    w.station_keeping_reward *= m[i];
    i += 1;
    w.gunner_base_reward *= m[i];
    i += 1;
    w.gunner_per_ammo *= m[i];
    i += 1;
    w.gunner_working_bonus *= m[i];
    i += 1;
    w.gunner_distance_factor *= m[i];
    i += 1;
    w.firefighter_base_reward *= m[i];
    i += 1;
    w.firefighter_distance_factor *= m[i];
    i += 1;
    w.healing_reward *= m[i];
    i += 1;
    w.sickbay_distance_factor *= m[i];
    i += 1;
    w.backtracking_penalty *= m[i];
    i += 1;
    w.solution_solver_reward *= m[i];
    i += 1;
    w.solution_distance_factor *= m[i];
    i += 1;
    w.situation_logistics_reward *= m[i];
    i += 1;
    w.situation_resolved_reward *= m[i];
    i += 1;
    w.ammo_stockpile_reward *= m[i];
    i += 1;
    w.loose_ammo_reward *= m[i];
    i += 1;
    w.hazard_proximity_reward *= m[i];
    i += 1;
    w.situation_exposure_penalty *= m[i];
    i += 1;
    w.system_disabled_penalty *= m[i];
    i += 1;
    w.shooting_reward *= m[i];
    i += 1;
    w.scavenger_reward *= m[i];
    i += 1;
    w.repair_proximity_reward *= m[i];
    i += 1;
    w.cargo_repair_incentive *= m[i];
    i += 1;
    w.boss_level_reward *= m[i];
    i += 1;
    w.turn_penalty *= m[i];
    i += 1;
    w.step_penalty *= m[i];
    i += 1;
    w.checkmate_threshold *= m[i];
    i += 1;
    w.checkmate_multiplier *= m[i];
    i += 1;
    w.critical_hull_threshold *= m[i];
    i += 1;
    w.critical_hull_penalty_base *= m[i];
    i += 1;
    w.critical_hull_penalty_per_hp *= m[i];
    i += 1;
    w.critical_fire_threshold = (w.critical_fire_threshold as f64 * m[i]) as usize;
    i += 1;
    w.critical_fire_penalty_per_token *= m[i];
    i += 1;
    w.hull_exponent *= m[i];
    i += 1;
    w.fire_exponent *= m[i];
    i += 1;
    w.cargo_repair_exponent *= m[i];
    i += 1;
    w.hull_risk_exponent *= m[i];
    i += 1;
    w.fire_urgency_mult *= m[i];
    i += 1;
    w.hazard_proximity_range *= m[i];
    i += 1;
    w.gunner_dist_range *= m[i];
    i += 1;
    w.gunner_per_ammo_mult *= m[i];
    i += 1;
    w.gunner_en_route_mult *= m[i];
    i += 1;
    w.gunner_wheelbarrow_penalty *= m[i];
    i += 1;
    w.baker_wheelbarrow_mult *= m[i];
    i += 1;
    w.threat_severe_reward *= m[i];
    i += 1;
    w.threat_mitigated_reward *= m[i];
    i += 1;
    w.threat_hull_risk_mult *= m[i];
    i += 1;
    w.threat_shield_waste_penalty *= m[i];

    w
}

pub fn apply_multipliers_rhea(base: &RheaScoringWeights, m: &[f64]) -> RheaScoringWeights {
    let mut w = base.clone();
    let mut i = 0;

    if m.len() < 10 {
        panic!(
            "Multiplier vector too short for Rhea weights. Expected 10, got {}",
            m.len()
        );
    }

    w.victory_base *= m[i];
    i += 1;
    w.victory_hull_mult *= m[i];
    i += 1;
    w.defeat_penalty *= m[i];
    i += 1;
    w.boss_damage_reward *= m[i];
    i += 1;

    // Threshold is integer
    w.hull_critical_threshold = (w.hull_critical_threshold as f64 * m[i]) as i32;
    i += 1;

    w.hull_critical_penalty_base *= m[i];
    i += 1;
    w.hull_normal_reward *= m[i];
    i += 1;
    w.fire_penalty *= m[i];
    i += 1;
    w.ammo_holding_reward *= m[i];
    i += 1;
    w.turn_penalty *= m[i];

    w
}

fn get_param_count(target: Target) -> usize {
    match target {
        Target::Beam => 59,
        Target::Rhea => 10,
    }
}

fn mutate(rng: &mut impl Rng, genome: &mut Vec<f64>) {
    let idx = rng.gen_range(0..genome.len());
    // Mutate by +/- 20% or random small noise
    if rng.gen_bool(0.5) {
        genome[idx] *= rng.gen_range(0.8..1.2);
    } else {
        // For multipliers, additive noise should be small
        genome[idx] += rng.gen_range(-0.1..0.1);
    }
    if genome[idx] < 0.0 {
        genome[idx] = 0.0;
    }
}

fn evaluate<F>(
    config: &OptimizerConfig,
    multipliers: &[f64],
    progress_callback: F,
) -> EvaluationMetrics
where
    F: Fn(SearchProgress) + Sync + Send + Clone,
{
    let metrics: EvaluationMetrics = config
        .seeds
        .par_iter()
        .map(|&seed| {
            let mut m = EvaluationMetrics::default();
            let mut fitness = 0.0;
            // Only report progress for the first seed to avoid TUI spam
            // Or maybe report all but filtered by index upstream?
            // Reporting all seeds for one individual is fine, the TUI will just jitter rapidly.
            // Let's report only for the first seed for stability in this view.
            let cb = if seed == config.seeds[0] {
                Some(progress_callback.clone())
            } else {
                None
            };

            let sol = match config.target {
                Target::Beam => {
                    let weights = apply_multipliers_beam(&get_base_weights_beam(), multipliers);
                    let search_config = BeamSearchConfig {
                        players: 6,
                        seed,
                        width: 100, // Reduced width for speed during optimization
                        steps: 1000,
                        time_limit: 5,
                        verbose: false,
                    };
                    beam_search(&search_config, &weights, cb)
                }
                Target::Rhea => {
                    let weights = apply_multipliers_rhea(&get_base_weights_rhea(), multipliers);
                    let search_config = RHEAConfig {
                        players: 6,
                        seed,
                        horizon: config.rhea_horizon,
                        generations: config.rhea_generations,
                        population_size: config.rhea_population,
                        max_steps: 1000,
                        time_limit: 5,
                        verbose: false,
                    };
                    rhea_search(&search_config, &weights) // RHEA doesn't support progress callback yet
                }
            };

            if let Some(sol) = sol {
                // 1. Victory (Ultimate Goal)
                if sol.state.phase == GamePhase::Victory {
                    fitness += 100_000.0;
                    fitness += (200 - sol.state.turn_count as i32).max(0) as f64 * 100.0;
                    m.wins = 1;
                } else if sol.state.phase == GamePhase::GameOver {
                    m.losses = 1;
                    fitness -= 10_000.0;
                } else {
                    m.timeouts = 1; // Stuck or time limit
                }

                // 2. Boss Progress (Major Milestones)
                fitness += (sol.state.boss_level as f64) * 10_000.0;

                // 3. Current Boss Damage (Progress within level)
                let damage_dealt = (sol.state.enemy.max_hp - sol.state.enemy.hp) as f64;
                fitness += damage_dealt * 200.0;

                // 4. Hull Integrity (Survival)
                if sol.state.hull_integrity > 0 {
                    fitness += sol.state.hull_integrity as f64 * 300.0;
                } else {
                    // Already handled in GameOver check logic if needed, but keeping for scoring
                }

                // 5. Situation Control (Clean Board)
                fitness -= sol.state.active_situations.len() as f64 * 500.0;

                // 6. Hazard Control
                let hazard_count: usize =
                    sol.state.map.rooms.values().map(|r| r.hazards.len()).sum();
                fitness -= hazard_count as f64 * 100.0;

                // 7. Survival Duration (Secondary)
                if sol.state.phase != GamePhase::GameOver {
                    fitness += sol.state.turn_count as f64 * 20.0;
                }

                m.score = fitness;
            } else {
                m.panics = 1;
                m.score = -20_000.0; // Failed to find any path
            }
            m
        })
        .reduce(EvaluationMetrics::default, |mut a, b| {
            a.add(&b);
            a
        });

    let mut final_metrics = metrics;
    final_metrics.average(config.seeds.len());
    final_metrics
}

pub fn run_ga(config: &OptimizerConfig, tx: Sender<OptimizerMessage>) {
    let param_count = get_param_count(config.target);
    let mut rng = rand::thread_rng();

    // Initialize Population
    let default_genome = vec![1.0; param_count];
    let mut population: Vec<Vec<f64>> = (0..config.population)
        .map(|i| {
            if i == 0 {
                default_genome.clone()
            } else {
                let mut g = default_genome.clone();
                for _ in 0..3 {
                    mutate(&mut rng, &mut g);
                }
                g
            }
        })
        .collect();

    for gen in 0..config.generations {
        // Evaluate
        let mut scored_pop: Vec<(EvaluationMetrics, Vec<f64>)> = population
            .par_iter()
            .enumerate()
            .map_with(tx.clone(), |tx, (i, genome)| {
                let current_tx = tx.clone();
                let metrics = evaluate(config, genome, move |p: SearchProgress| {
                    let _ = current_tx.send(OptimizerMessage::IndividualUpdate {
                        generation: gen,
                        index: i,
                        progress: p,
                    });
                });
                let _ = tx.send(OptimizerMessage::IndividualDone {
                    generation: gen,
                    index: i,
                    score: metrics.score,
                    metrics: metrics.clone(),
                });
                (metrics, genome.clone())
            })
            .collect();

        // Sort descending by score
        scored_pop.sort_by(|a, b| b.0.score.partial_cmp(&a.0.score).unwrap());

        let best_metrics = scored_pop[0].0.clone();
        let best_score = best_metrics.score;
        let avg_score: f64 =
            scored_pop.iter().map(|s| s.0.score).sum::<f64>() / (config.population as f64);
        let best_genome = scored_pop[0].1.clone();

        let weights_beam = if config.target == Target::Beam {
            Some(apply_multipliers_beam(
                &get_base_weights_beam(),
                &best_genome,
            ))
        } else {
            None
        };
        let weights_rhea = if config.target == Target::Rhea {
            Some(apply_multipliers_rhea(
                &get_base_weights_rhea(),
                &best_genome,
            ))
        } else {
            None
        };

        let _ = tx.send(OptimizerMessage::GenerationDone(OptimizationStatus {
            generation: gen,
            best_score,
            avg_score,
            best_metrics,
            best_genome: best_genome.clone(),
            current_weights_beam: weights_beam,
            current_weights_rhea: weights_rhea,
        }));

        // Elitism: Keep top 20%
        let elite_count = (config.population as f64 * 0.2).ceil() as usize;
        let mut new_pop = Vec::new();
        for i in 0..elite_count {
            new_pop.push(scored_pop[i].1.clone());
        }

        // Offspring
        while new_pop.len() < config.population {
            // Tournament Selection
            let p1 = &scored_pop[rng.gen_range(0..elite_count * 2.min(config.population))];
            let p2 = &scored_pop[rng.gen_range(0..elite_count * 2.min(config.population))];

            // Crossover
            let split = rng.gen_range(0..p1.1.len());
            let mut child = Vec::new();
            child.extend_from_slice(&p1.1[0..split]);
            child.extend_from_slice(&p2.1[split..]);

            // Mutation
            if rng.gen_bool(0.3) {
                mutate(&mut rng, &mut child);
            }

            new_pop.push(child);
        }

        population = new_pop;
    }
}

pub fn run_spsa(config: &OptimizerConfig, tx: Sender<OptimizerMessage>) {
    let param_count = get_param_count(config.target);

    // Start with identity multipliers
    let mut theta = vec![1.0; param_count];
    let p = theta.len();

    // SPSA hyperparameters
    let c = 0.05;
    let gamma = 0.101;
    let a = 0.1;
    let big_a = 20.0;
    let alpha = 0.602;

    let mut best_theta = theta.clone();
    let mut current_metrics = evaluate(config, &theta, |_| {});
    let mut best_metrics = current_metrics.clone();

    for k in 0..config.generations {
        let mut rng = rand::thread_rng();

        let ak = a / (k as f64 + 1.0 + big_a).powf(alpha);
        let ck = c / (k as f64 + 1.0).powf(gamma);

        // Generate Bernoulli perturbation vector (+1 or -1)
        let delta: Vec<f64> = (0..p)
            .map(|_| if rng.gen_bool(0.5) { 1.0 } else { -1.0 })
            .collect();

        // Theta + ck * delta
        let mut theta_plus = theta.clone();
        for i in 0..p {
            theta_plus[i] += ck * delta[i];
            if theta_plus[i] < 0.0 {
                theta_plus[i] = 0.0;
            }
        }

        // Theta - ck * delta
        let mut theta_minus = theta.clone();
        for i in 0..p {
            theta_minus[i] -= ck * delta[i];
            if theta_minus[i] < 0.0 {
                theta_minus[i] = 0.0;
            }
        }

        let m_plus = evaluate(config, &theta_plus, |_| {});
        let m_minus = evaluate(config, &theta_minus, |_| {});

        // Gradient Estimate
        let mut ghat = vec![0.0; p];
        for i in 0..p {
            ghat[i] = (m_plus.score - m_minus.score) / (2.0 * ck * delta[i]);
        }

        // Update Theta (Gradient Ascent)
        for i in 0..p {
            theta[i] += ak * ghat[i];
            if theta[i] < 0.0 {
                theta[i] = 0.0;
            }
        }

        current_metrics = evaluate(config, &theta, |p| {
            // Optional: Send update for SPSA current best?
            // Since SPSA is single-point, 'index' is effectively 0.
            let _ = tx.send(OptimizerMessage::IndividualUpdate {
                generation: k,
                index: 0,
                progress: p,
            });
        });

        if current_metrics.score > best_metrics.score {
            best_metrics = current_metrics.clone();
            best_theta = theta.clone();
        }

        let weights_beam = if config.target == Target::Beam {
            Some(apply_multipliers_beam(
                &get_base_weights_beam(),
                &best_theta,
            ))
        } else {
            None
        };
        let weights_rhea = if config.target == Target::Rhea {
            Some(apply_multipliers_rhea(
                &get_base_weights_rhea(),
                &best_theta,
            ))
        } else {
            None
        };

        let _ = tx.send(OptimizerMessage::GenerationDone(OptimizationStatus {
            generation: k,
            best_score: best_metrics.score,
            avg_score: current_metrics.score,
            best_metrics: best_metrics.clone(),
            best_genome: best_theta.clone(),
            current_weights_beam: weights_beam,
            current_weights_rhea: weights_rhea,
        }));
    }
}
