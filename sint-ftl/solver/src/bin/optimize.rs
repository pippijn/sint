use clap::{Parser, ValueEnum};
use rand::prelude::*;
use rayon::prelude::*;
use sint_core::types::GamePhase;
use sint_solver::scoring::ScoringWeights;
use sint_solver::search::{beam_search, SearchConfig};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Optimization Strategy
    #[arg(long, value_enum, default_value_t = Strategy::GA)]
    strategy: Strategy,

    /// Generations / Iterations
    #[arg(short, long, default_value_t = 20)]
    generations: usize,

    /// Population Size (for GA)
    #[arg(short, long, default_value_t = 40)]
    population: usize,

    /// Seeds to evaluate (comma separated)
    #[arg(long, default_value = "12345")]
    seeds: String,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Strategy {
    GA,
    SPSA,
}

const PARAM_COUNT: usize = 23;

fn get_base_weights() -> ScoringWeights {
    ScoringWeights::default()
}

fn apply_multipliers(base: &ScoringWeights, m: &[f64]) -> ScoringWeights {
    let mut w = base.clone();
    w.hull_integrity *= m[0];
    w.enemy_hp *= m[1];
    w.player_hp *= m[2];
    w.ap_balance *= m[3];
    w.fire_penalty_base *= m[4];
    w.water_penalty *= m[5];
    w.active_situation_penalty *= m[6];
    w.threat_player_penalty *= m[7];
    w.threat_system_penalty *= m[8];
    w.death_penalty *= m[9];
    w.gunner_base_reward *= m[10];
    w.gunner_per_ammo *= m[11];
    w.gunner_working_bonus *= m[12];
    w.gunner_distance_factor *= m[13];
    w.firefighter_base_reward *= m[14];
    w.firefighter_distance_factor *= m[15];
    w.healing_reward *= m[16];
    w.sickbay_distance_factor *= m[17];
    w.backtracking_penalty *= m[18];
    w.solution_solver_reward *= m[19];
    w.solution_distance_factor *= m[20];
    w.ammo_stockpile_reward *= m[21];
    w.boss_level_reward *= m[22];
    w
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

fn evaluate(base: &ScoringWeights, multipliers: &[f64], seeds: &[u64]) -> f64 {
    let weights = apply_multipliers(base, multipliers);
    let total_score: f64 = seeds
        .par_iter()
        .map(|&seed| {
            let config = SearchConfig {
                players: 6,
                seed,
                width: 100, // Small width for speed
                steps: 100, // Short horizon
                time_limit: 5,
                verbose: false,
            };

            if let Some(sol) = beam_search(&config, &weights) {
                let mut fitness = 0.0;

                // 1. Victory (Ultimate Goal)
                if sol.state.phase == GamePhase::Victory {
                    fitness += 100_000.0;
                    // Speed bonus
                    fitness += (200 - sol.state.turn_count as i32).max(0) as f64 * 100.0;
                }

                // 2. Boss Progress (Major Milestones)
                fitness += (sol.state.boss_level as f64) * 10_000.0;

                // 3. Current Boss Damage (Progress within level)
                // We want to kill the current boss.
                let damage_dealt = (sol.state.enemy.max_hp - sol.state.enemy.hp) as f64;
                fitness += damage_dealt * 200.0;

                // 4. Hull Integrity (Survival)
                // If dead, hull is <= 0.
                if sol.state.hull_integrity > 0 {
                    fitness += sol.state.hull_integrity as f64 * 300.0;
                } else {
                    // Massive penalty for death
                    fitness -= 10_000.0;
                }

                // 5. Situation Control (Clean Board)
                fitness -= sol.state.active_situations.len() as f64 * 500.0;

                // 6. Hazard Control
                let hazard_count: usize =
                    sol.state.map.rooms.values().map(|r| r.hazards.len()).sum();
                fitness -= hazard_count as f64 * 100.0;

                // 7. Survival Duration (Secondary)
                // Only reward rounds if we are alive
                if sol.state.phase != GamePhase::GameOver {
                    fitness += sol.state.turn_count as f64 * 20.0;
                }

                fitness
            } else {
                -20_000.0 // Failed to find any path
            }
        })
        .sum();

    total_score / (seeds.len() as f64)
}

fn run_ga(args: &Args, seeds: &[u64]) {
    println!(
        "🧬 Starting Evolution: Gens={}, Pop={}, Seeds={:?}",
        args.generations, args.population, seeds
    );

    let base_weights = get_base_weights();
    let mut rng = rand::thread_rng();

    // Initialize Population with Multipliers (start at 1.0)
    let default_genome = vec![1.0; PARAM_COUNT];
    let mut population: Vec<Vec<f64>> = (0..args.population)
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

    for gen in 0..args.generations {
        println!("\n--- Generation {} ---", gen);

        // Evaluate
        let mut scored_pop: Vec<(f64, Vec<f64>)> = population
            .par_iter()
            .map(|genome| {
                let score = evaluate(&base_weights, genome, seeds);
                (score, genome.clone())
            })
            .collect();

        // Sort descending
        scored_pop.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        let best_score = scored_pop[0].0;
        let avg_score: f64 = scored_pop.iter().map(|s| s.0).sum::<f64>() / (args.population as f64);

        println!("Best: {:.2} | Avg: {:.2}", best_score, avg_score);

        // Elitism: Keep top 20%
        let elite_count = (args.population as f64 * 0.2).ceil() as usize;
        let mut new_pop = Vec::new();
        for i in 0..elite_count {
            new_pop.push(scored_pop[i].1.clone());
        }

        // Offspring
        while new_pop.len() < args.population {
            // Tournament Selection
            let p1 = &scored_pop[rng.gen_range(0..elite_count * 2.min(args.population))];
            let p2 = &scored_pop[rng.gen_range(0..elite_count * 2.min(args.population))];

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

    // Print Best
    let best_genome = &population[0];
    let best_weights = apply_multipliers(&base_weights, best_genome);
    println!("\n🏆 Best Weights Found:");
    println!("{:#?}", best_weights);
}

fn run_spsa(args: &Args, seeds: &[u64]) {
    println!(
        "📉 Starting SPSA: Iterations={}, Seeds={:?}",
        args.generations, seeds
    );

    let base_weights = get_base_weights();

    // Start with identity multipliers
    let mut theta = vec![1.0; PARAM_COUNT];
    let p = theta.len();

    // SPSA hyperparameters (Tuned for normalized multipliers ~1.0)
    let c = 0.05; // Perturbation size (5%)
    let gamma = 0.101;
    let a = 0.1; // Step size scaling (since params are ~1.0, steps should be small)
    let big_a = 20.0; // Stability constant
    let alpha = 0.602;

    let mut best_theta = theta.clone();
    let mut best_score = evaluate(&base_weights, &theta, seeds);

    println!("Initial Score: {:.2}", best_score);

    for k in 0..args.generations {
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
            } // Constraint
        }

        // Theta - ck * delta
        let mut theta_minus = theta.clone();
        for i in 0..p {
            theta_minus[i] -= ck * delta[i];
            if theta_minus[i] < 0.0 {
                theta_minus[i] = 0.0;
            } // Constraint
        }

        let y_plus = evaluate(&base_weights, &theta_plus, seeds);
        let y_minus = evaluate(&base_weights, &theta_minus, seeds);

        // Gradient Estimate
        // g_k = (y_plus - y_minus) / (2 * ck * delta)
        let mut ghat = vec![0.0; p];
        for i in 0..p {
            // Note: delta is +/- 1.0
            ghat[i] = (y_plus - y_minus) / (2.0 * ck * delta[i]);
        }

        // Update Theta (Gradient Ascent)
        for i in 0..p {
            theta[i] += ak * ghat[i];
            if theta[i] < 0.0 {
                theta[i] = 0.0;
            }
        }

        let current_score = evaluate(&base_weights, &theta, seeds);
        println!(
            "Iter {}: Score {:.2} (Best {:.2}) | ak={:.4} ck={:.4}",
            k, current_score, best_score, ak, ck
        );

        if current_score > best_score {
            best_score = current_score;
            best_theta = theta.clone();
        }
    }

    println!("\n🏆 Best Weights Found (SPSA):");
    println!("{:#?}", apply_multipliers(&base_weights, &best_theta));
}

fn main() {
    let args = Args::parse();
    let seeds: Vec<u64> = args
        .seeds
        .split(',')
        .map(|s| s.trim().parse().unwrap())
        .collect();

    match args.strategy {
        Strategy::GA => run_ga(&args, &seeds),
        Strategy::SPSA => run_spsa(&args, &seeds),
    }
}
