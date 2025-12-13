use clap::{Parser, ValueEnum};
use rand::prelude::*;
use rayon::prelude::*;
use sint_core::types::GamePhase;
use sint_solver::scoring::beam::BeamScoringWeights;
use sint_solver::scoring::rhea::RheaScoringWeights;
use sint_solver::search::beam::{beam_search, BeamSearchConfig};
use sint_solver::search::rhea::{rhea_search, RHEAConfig};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Optimization Strategy
    #[arg(long, value_enum, default_value_t = Strategy::GA)]
    strategy: Strategy,

    /// Target Search Algorithm
    #[arg(long, value_enum, default_value_t = Target::Beam)]
    target: Target,

    /// Generations / Iterations (Optimizer)
    #[arg(short, long, default_value_t = 20)]
    generations: usize,

    /// Population Size (Optimizer GA)
    #[arg(short, long, default_value_t = 40)]
    population: usize,

    /// Seeds to evaluate (comma separated)
    #[arg(long, default_value = "12345")]
    seeds: String,

    // --- RHEA Specifics ---
    /// RHEA Horizon
    #[arg(long, default_value_t = 10)]
    rhea_horizon: usize,

    /// RHEA Generations (per search step)
    #[arg(long, default_value_t = 50)]
    rhea_generations: usize,

    /// RHEA Population (per search step)
    #[arg(long, default_value_t = 20)]
    rhea_population: usize,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Strategy {
    GA,
    SPSA,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Target {
    Beam,
    Rhea,
}

fn get_base_weights_beam() -> BeamScoringWeights {
    BeamScoringWeights::default()
}

fn get_base_weights_rhea() -> RheaScoringWeights {
    RheaScoringWeights::default()
}

// Map multipliers vector to BeamScoringWeights
fn apply_multipliers_beam(base: &BeamScoringWeights, m: &[f64]) -> BeamScoringWeights {
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

fn apply_multipliers_rhea(base: &RheaScoringWeights, m: &[f64]) -> RheaScoringWeights {
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

fn evaluate(args: &Args, multipliers: &[f64], seeds: &[u64]) -> f64 {
    let total_score: f64 = seeds
        .par_iter()
        .map(|&seed| {
            let mut fitness = 0.0;
            let sol = match args.target {
                Target::Beam => {
                    let weights = apply_multipliers_beam(&get_base_weights_beam(), multipliers);
                    let config = BeamSearchConfig {
                        players: 6,
                        seed,
                        width: 100, // Reduced width for speed during optimization
                        steps: 1000,
                        time_limit: 5,
                        verbose: false,
                    };
                    beam_search(&config, &weights)
                }
                Target::Rhea => {
                    let weights = apply_multipliers_rhea(&get_base_weights_rhea(), multipliers);
                    let config = RHEAConfig {
                        players: 6,
                        seed,
                        horizon: args.rhea_horizon,
                        generations: args.rhea_generations,
                        population_size: args.rhea_population,
                        max_steps: 1000,
                        time_limit: 5,
                        verbose: false,
                    };
                    rhea_search(&config, &weights)
                }
            };

            if let Some(sol) = sol {
                // 1. Victory (Ultimate Goal)
                if sol.state.phase == GamePhase::Victory {
                    fitness += 100_000.0;
                    // Speed bonus
                    fitness += (200 - sol.state.turn_count as i32).max(0) as f64 * 100.0;
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
                    fitness -= 10_000.0;
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
        "🧬 Starting Evolution ({}): Gens={}, Pop={}, Seeds={:?}",
        match args.target {
            Target::Beam => "Beam",
            Target::Rhea => "Rhea",
        },
        args.generations,
        args.population,
        seeds
    );

    let param_count = get_param_count(args.target);
    let mut rng = rand::thread_rng();

    // Initialize Population with Multipliers (start at 1.0)
    let default_genome = vec![1.0; param_count];
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
                let score = evaluate(args, genome, seeds);
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
    println!("\n🏆 Best Weights Found:");
    match args.target {
        Target::Beam => println!(
            "{:#?}",
            apply_multipliers_beam(&get_base_weights_beam(), best_genome)
        ),
        Target::Rhea => println!(
            "{:#?}",
            apply_multipliers_rhea(&get_base_weights_rhea(), best_genome)
        ),
    }
}

fn run_spsa(args: &Args, seeds: &[u64]) {
    println!(
        "📉 Starting SPSA ({}): Iterations={}, Seeds={:?}",
        match args.target {
            Target::Beam => "Beam",
            Target::Rhea => "Rhea",
        },
        args.generations,
        seeds
    );

    let param_count = get_param_count(args.target);

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
    let mut best_score = evaluate(args, &theta, seeds);

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

        let y_plus = evaluate(args, &theta_plus, seeds);
        let y_minus = evaluate(args, &theta_minus, seeds);

        // Gradient Estimate
        let mut ghat = vec![0.0; p];
        for i in 0..p {
            ghat[i] = (y_plus - y_minus) / (2.0 * ck * delta[i]);
        }

        // Update Theta (Gradient Ascent)
        for i in 0..p {
            theta[i] += ak * ghat[i];
            if theta[i] < 0.0 {
                theta[i] = 0.0;
            }
        }

        let current_score = evaluate(args, &theta, seeds);
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
    match args.target {
        Target::Beam => println!(
            "{:#?}",
            apply_multipliers_beam(&get_base_weights_beam(), &best_theta)
        ),
        Target::Rhea => println!(
            "{:#?}",
            apply_multipliers_rhea(&get_base_weights_rhea(), &best_theta)
        ),
    }
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
