use crate::scoring::beam::BeamScoringWeights;
use crate::scoring::rhea::RheaScoringWeights;
use crate::search::SearchProgress;
use crate::search::beam::beam_search;
use crate::search::config::{BeamSearchConfig, RHEAConfig};
use crate::search::rhea::rhea_search;
use dashmap::DashMap;
use rand::prelude::*;
use sint_core::types::GamePhase;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub enum OptimizerMessage {
    GenerationDone(Box<OptimizationStatus>),
    IndividualStarting {
        generation: usize,
        index: usize,
        genome: Vec<f64>,
    },
    IndividualDone {
        generation: usize,
        index: usize,
        score: f64,
        metrics: EvaluationMetrics,
        genome: Vec<f64>,
    },
    IndividualUpdate {
        generation: usize,
        index: usize,
        seed_idx: usize,
        progress: SearchProgress,
        score_history: Vec<f32>,
    },
    SeedDone {
        generation: usize,
        index: usize,
        seed_idx: usize,
        status: u8,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Strategy {
    GA,
    Spsa,
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
    pub beam_width: usize,
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
    pub panics: usize,
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
        }
    }
}

#[derive(Clone, Debug)]
pub struct OptimizationStatus {
    pub generation: usize,
    pub best_score: f64,
    pub avg_score: f64,
    pub best_metrics: EvaluationMetrics,
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

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

pub fn apply_multipliers<T: Serialize + DeserializeOwned>(base: &T, m: &[f64]) -> T {
    let mut map: Map<String, Value> =
        serde_json::from_value(serde_json::to_value(base).unwrap()).unwrap();

    for (i, val) in map.values_mut().enumerate() {
        if i >= m.len() {
            break;
        }

        if val.is_u64() {
            let f = val.as_f64().unwrap();
            *val = Value::from((f * m[i]).round() as u64);
        } else if val.is_i64() {
            let f = val.as_f64().unwrap();
            *val = Value::from((f * m[i]).round() as i64);
        } else if let Some(f) = val.as_f64() {
            *val = Value::from(f * m[i]);
        }
    }

    serde_json::from_value(Value::Object(map)).unwrap()
}

pub fn get_param_names<T: Serialize + Default>() -> Vec<String> {
    let base = T::default();
    let map: Map<String, Value> =
        serde_json::from_value(serde_json::to_value(base).unwrap()).unwrap();
    map.keys().cloned().collect()
}

pub fn apply_multipliers_beam(base: &BeamScoringWeights, m: &[f64]) -> BeamScoringWeights {
    apply_multipliers(base, m)
}

pub fn apply_multipliers_rhea(base: &RheaScoringWeights, m: &[f64]) -> RheaScoringWeights {
    apply_multipliers(base, m)
}

fn get_param_count(target: Target) -> usize {
    match target {
        Target::Beam => get_param_names::<BeamScoringWeights>().len(),
        Target::Rhea => get_param_names::<RheaScoringWeights>().len(),
    }
}

fn mutate(rng: &mut impl Rng, genome: &mut [f64]) {
    let mutation_rate = 1.0 / genome.len() as f64;

    for val in genome.iter_mut() {
        if rng.random_bool(mutation_rate.max(0.1)) {
            if rng.random_bool(0.9) {
                let noise: f64 = rng.sample(rand_distr::StandardNormal);
                *val += noise * 0.1 * (*val).max(1.0);
            } else {
                *val = rng.random_range(0.0..5.0);
            }

            if *val < 0.0 {
                *val = 0.0;
            }
        }
    }
}

struct GameTask {
    genome_idx: usize,
    seed_idx: usize,
    genome: Vec<f64>,
    seed: u64,
}

struct GameResult {
    genome_idx: usize,
    metrics: EvaluationMetrics,
}

fn evaluate_batch(
    config: &OptimizerConfig,
    genomes: &[Vec<f64>],
    tx: &Sender<OptimizerMessage>,
    generation: usize,
) -> Vec<EvaluationMetrics> {
    let mut tasks = Vec::new();
    for (g_idx, genome) in genomes.iter().enumerate() {
        for (s_idx, &seed) in config.seeds.iter().enumerate() {
            tasks.push(GameTask {
                genome_idx: g_idx,
                seed_idx: s_idx,
                genome: genome.clone(),
                seed,
            });
        }
    }

    let num_cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    // Limit parallel games to avoid oversubscription, but allow them to use nested Rayon parallelism.
    let max_parallel_games = (num_cores / 4).max(1);
    let active_games = Arc::new(AtomicUsize::new(0));
    let histories: Arc<DashMap<(usize, usize), Vec<f32>>> = Arc::new(DashMap::new());

    let (result_tx, result_rx) = crossbeam_channel::unbounded::<GameResult>();
    let mut metrics_per_genome = vec![EvaluationMetrics::default(); genomes.len()];
    let total_tasks = tasks.len();

    rayon::scope(|s| {
        for task in tasks {
            // Cooperative yield while waiting for a slot.
            // This thread will help with other Rayon tasks (like population evaluation) instead of blocking.
            while active_games.load(Ordering::Acquire) >= max_parallel_games {
                rayon::yield_now();
            }

            active_games.fetch_add(1, Ordering::SeqCst);
            let active_counter = active_games.clone();
            let opt_tx = tx.clone();
            let res_tx = result_tx.clone();
            let histories_ref = histories.clone();
            let config_clone = config.clone();

            s.spawn(move |_| {
                let mut m = EvaluationMetrics::default();
                let mut fitness = 0.0;

                let progress_tx = opt_tx.clone();
                let hist_ref = histories_ref.clone();
                let g_idx = task.genome_idx;
                let s_idx = task.seed_idx;

                let cb = move |p: SearchProgress| {
                    let hull_part = p.node.state.hull_integrity as f32 / 20.0;
                    let score_part = (p.node.score.total / 100_000.0).min(1.0) as f32;
                    let health = (hull_part * 0.7 + score_part * 0.3).clamp(0.0, 1.0);

                    let round = p.node.state.turn_count as usize;
                    let mut history = hist_ref.entry((g_idx, s_idx)).or_default();
                    if round >= history.len() {
                        history.push(health);
                    } else {
                        history[round] = health;
                    }
                    let history_snapshot = history.clone();
                    drop(history);

                    let _ = progress_tx.send(OptimizerMessage::IndividualUpdate {
                        generation,
                        index: g_idx,
                        seed_idx: s_idx,
                        progress: p,
                        score_history: history_snapshot,
                    });
                };

                let sol = match config_clone.target {
                    Target::Beam => {
                        let weights =
                            apply_multipliers_beam(&get_base_weights_beam(), &task.genome);
                        let search_config = BeamSearchConfig {
                            players: 6,
                            seed: task.seed,
                            width: config_clone.beam_width,
                            steps: 3000,
                            time_limit: 300,
                            verbose: false,
                        };
                        beam_search(&search_config, &weights, Some(cb))
                    }
                    Target::Rhea => {
                        let weights =
                            apply_multipliers_rhea(&get_base_weights_rhea(), &task.genome);
                        let search_config = RHEAConfig {
                            players: 6,
                            seed: task.seed,
                            horizon: config_clone.rhea_horizon,
                            generations: config_clone.rhea_generations,
                            population_size: config_clone.rhea_population,
                            max_steps: 3000,
                            time_limit: 300,
                            verbose: false,
                        };
                        rhea_search(&search_config, &weights, Some(cb))
                    }
                };

                let status;
                if let Some(sol) = sol {
                    if sol.state.phase == GamePhase::Victory {
                        fitness += 100_000.0;
                        fitness += (200 - sol.state.turn_count as i32).max(0) as f64 * 100.0;
                        m.wins = 1;
                        status = 2;
                    } else if sol.state.phase == GamePhase::GameOver {
                        m.losses = 1;
                        fitness -= 10_000.0;
                        status = 3;
                    } else {
                        m.timeouts = 1;
                        status = 5;
                    }
                    fitness += (sol.state.boss_level as f64) * 10_000.0;
                    let damage_dealt = (sol.state.enemy.max_hp - sol.state.enemy.hp) as f64;
                    fitness += damage_dealt * 200.0;
                    if sol.state.hull_integrity > 0 {
                        fitness += sol.state.hull_integrity as f64 * 300.0;
                    }
                    fitness -= sol.state.active_situations.len() as f64 * 500.0;
                    let hazard_count: usize =
                        sol.state.map.rooms.values().map(|r| r.hazards.len()).sum();
                    fitness -= hazard_count as f64 * 100.0;
                    if sol.state.phase != GamePhase::GameOver {
                        fitness += sol.state.turn_count as f64 * 20.0;
                    }
                    m.score = fitness;
                } else {
                    m.panics = 1;
                    m.score = -20_000.0;
                    status = 4;
                }

                let _ = opt_tx.send(OptimizerMessage::SeedDone {
                    generation,
                    index: g_idx,
                    seed_idx: s_idx,
                    status,
                });

                let _ = res_tx.send(GameResult {
                    genome_idx: task.genome_idx,
                    metrics: m,
                });

                active_counter.fetch_sub(1, Ordering::SeqCst);
            });
        }
    });

    // Collect all results
    for _ in 0..total_tasks {
        if let Ok(res) = result_rx.recv() {
            metrics_per_genome[res.genome_idx].add(&res.metrics);
        }
    }

    metrics_per_genome
}

pub fn run_ga(config: &OptimizerConfig, tx: Sender<OptimizerMessage>) {
    let param_count = get_param_count(config.target);
    let mut rng = rand::rng();

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

    for generation in 0..config.generations {
        for (i, genome) in population.iter().enumerate() {
            let _ = tx.send(OptimizerMessage::IndividualStarting {
                generation,
                index: i,
                genome: genome.clone(),
            });
        }

        let scored_metrics = evaluate_batch(config, &population, &tx, generation);

        for (i, m) in scored_metrics.iter().enumerate() {
            let _ = tx.send(OptimizerMessage::IndividualDone {
                generation,
                index: i,
                score: m.score / config.seeds.len() as f64,
                metrics: m.clone(),
                genome: population[i].clone(),
            });
        }

        let mut scored_pop: Vec<(EvaluationMetrics, Vec<f64>)> = scored_metrics
            .into_iter()
            .zip(population.into_iter())
            .collect();

        for (m, _) in scored_pop.iter_mut() {
            m.average(config.seeds.len());
        }

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

        let _ = tx.send(OptimizerMessage::GenerationDone(Box::new(
            OptimizationStatus {
                generation,
                best_score,
                avg_score,
                best_metrics,
                best_genome: best_genome.clone(),
                current_weights_beam: weights_beam,
                current_weights_rhea: weights_rhea,
            },
        )));

        let elite_count = (config.population as f64 * 0.2).ceil() as usize;
        let mut new_pop = Vec::new();
        for item in scored_pop.iter().take(elite_count) {
            new_pop.push(item.1.clone());
        }

        let total_rank_sum: usize = (1..=config.population).sum();

        // 1. Generate ALL children first
        let mut all_children = Vec::new();
        let mut pairings = Vec::new(); // Store which parents produced which children

        while new_pop.len() + all_children.len() < config.population {
            let select_parent_idx = |rng: &mut ThreadRng| {
                let mut r = rng.random_range(0..total_rank_sum);
                for i in 0..scored_pop.len() {
                    let weight = config.population - i;
                    if r < weight {
                        return i;
                    }
                    r -= weight;
                }
                0
            };

            let i1 = select_parent_idx(&mut rng);
            let i2 = select_parent_idx(&mut rng);
            let p1_genome = &scored_pop[i1].1;
            let p2_genome = &scored_pop[i2].1;

            let alpha = 0.5;
            let mut c1 = Vec::with_capacity(param_count);
            let mut c2 = Vec::with_capacity(param_count);
            for i in 0..param_count {
                let v1 = p1_genome[i];
                let v2 = p2_genome[i];
                let min = v1.min(v2);
                let max = v1.max(v2);
                let range = max - min;
                let lower = min - range * alpha;
                let upper = max + range * alpha;

                let mut val1 = rng.random_range(lower..=upper);
                if val1 < 0.0 {
                    val1 = 0.0;
                }
                c1.push(val1);

                let mut val2 = rng.random_range(lower..=upper);
                if val2 < 0.0 {
                    val2 = 0.0;
                }
                c2.push(val2);
            }

            mutate(&mut rng, &mut c1);
            mutate(&mut rng, &mut c2);

            let c1_idx = all_children.len();
            all_children.push(c1);
            let c2_idx = all_children.len();
            all_children.push(c2);

            pairings.push((i1, i2, c1_idx, c2_idx));
        }

        // 2. Evaluate all children in one large batch
        let children_metrics = evaluate_batch(config, &all_children, &tx, generation);

        // 3. Survival logic (Deterministic Crowding)
        for (i1, i2, c1_idx, c2_idx) in pairings {
            let p1 = &scored_pop[i1];
            let p2 = &scored_pop[i2];
            let c1_genome = &all_children[c1_idx];
            let c2_genome = &all_children[c2_idx];

            let mut m_c1 = children_metrics[c1_idx].clone();
            m_c1.average(config.seeds.len());
            let mut m_c2 = children_metrics[c2_idx].clone();
            m_c2.average(config.seeds.len());

            let dist = |g1: &[f64], g2: &[f64]| {
                g1.iter()
                    .zip(g2.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
            };

            if dist(&p1.1, c1_genome) + dist(&p2.1, c2_genome)
                < dist(&p1.1, c2_genome) + dist(&p2.1, c1_genome)
            {
                if m_c1.score > p1.0.score {
                    new_pop.push(c1_genome.clone());
                } else {
                    new_pop.push(p1.1.clone());
                }
                if new_pop.len() < config.population {
                    if m_c2.score > p2.0.score {
                        new_pop.push(c2_genome.clone());
                    } else {
                        new_pop.push(p2.1.clone());
                    }
                }
            } else {
                if m_c2.score > p1.0.score {
                    new_pop.push(c2_genome.clone());
                } else {
                    new_pop.push(p1.1.clone());
                }
                if new_pop.len() < config.population {
                    if m_c1.score > p2.0.score {
                        new_pop.push(c1_genome.clone());
                    } else {
                        new_pop.push(p2.1.clone());
                    }
                }
            }
        }

        population = new_pop;
    }
}

pub fn run_spsa(config: &OptimizerConfig, tx: Sender<OptimizerMessage>) {
    let param_count = get_param_count(config.target);
    let mut theta = vec![1.0; param_count];
    let p = theta.len();

    let c = 0.05;
    let gamma = 0.101;
    let a = 0.1;
    let big_a = 20.0;
    let alpha = 0.602;

    let mut best_theta = theta.clone();
    let initial_metrics_batch = evaluate_batch(config, &[theta.clone()], &tx, 0);
    let mut current_metrics = initial_metrics_batch[0].clone();
    current_metrics.average(config.seeds.len());
    let mut best_metrics = current_metrics.clone();

    for k in 0..config.generations {
        let mut rng = rand::rng();
        let ak = a / (k as f64 + 1.0 + big_a).powf(alpha);
        let ck = c / (k as f64 + 1.0).powf(gamma);

        let delta: Vec<f64> = (0..p)
            .map(|_| if rng.random_bool(0.5) { 1.0 } else { -1.0 })
            .collect();

        let mut theta_plus = theta.clone();
        let mut theta_minus = theta.clone();
        for i in 0..p {
            theta_plus[i] += ck * delta[i];
            if theta_plus[i] < 0.0 {
                theta_plus[i] = 0.0;
            }
            theta_minus[i] -= ck * delta[i];
            if theta_minus[i] < 0.0 {
                theta_minus[i] = 0.0;
            }
        }

        let perturbed = vec![theta_plus, theta_minus];
        let m_perturbed = evaluate_batch(config, &perturbed, &tx, k);
        let m_plus = &m_perturbed[0];
        let m_minus = &m_perturbed[1];

        let mut ghat = vec![0.0; p];
        for i in 0..p {
            ghat[i] = (m_plus.score - m_minus.score) / (2.0 * ck * delta[i]);
        }

        for i in 0..p {
            theta[i] += ak * ghat[i];
            if theta[i] < 0.0 {
                theta[i] = 0.0;
            }
        }

        let updated_metrics_batch = evaluate_batch(config, &[theta.clone()], &tx, k);
        current_metrics = updated_metrics_batch[0].clone();
        current_metrics.average(config.seeds.len());

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

        let _ = tx.send(OptimizerMessage::GenerationDone(Box::new(
            OptimizationStatus {
                generation: k,
                best_score: best_metrics.score,
                avg_score: current_metrics.score,
                best_metrics: best_metrics.clone(),
                best_genome: best_theta.clone(),
                current_weights_beam: weights_beam,
                current_weights_rhea: weights_rhea,
            },
        )));
    }
}
