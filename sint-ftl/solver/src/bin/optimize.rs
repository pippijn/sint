use clap::Parser;
use rand::prelude::*;
use rayon::prelude::*;
use sint_core::types::GamePhase;
use sint_solver::scoring::ScoringWeights;
use sint_solver::search::{beam_search, SearchConfig};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Generations
    #[arg(short, long, default_value_t = 20)]
    generations: usize,

    /// Population Size
    #[arg(short, long, default_value_t = 40)]
    population: usize,

    /// Seeds to evaluate (comma separated)
    #[arg(long, default_value = "12345,67890,11223")]
    seeds: String,
}

// Map ScoringWeights to a flat vector for GA
fn weights_to_vec(w: &ScoringWeights) -> Vec<f64> {
    vec![
        w.hull_integrity,
        w.enemy_hp,
        w.player_hp,
        w.ap_balance,
        w.fire_penalty_base,
        w.water_penalty,
        w.active_situation_penalty,
        w.threat_player_penalty,
        w.threat_system_penalty,
        w.death_penalty,
        w.gunner_base_reward,
        w.gunner_per_ammo,
        w.gunner_working_bonus,
        w.gunner_distance_factor,
        w.firefighter_base_reward,
        w.firefighter_distance_factor,
        w.baker_base_reward,
        w.baker_distance_factor,
        w.backtracking_penalty,
    ]
}

fn vec_to_weights(v: &[f64]) -> ScoringWeights {
    ScoringWeights {
        hull_integrity: v[0],
        enemy_hp: v[1],
        player_hp: v[2],
        ap_balance: v[3],
        fire_penalty_base: v[4],
        water_penalty: v[5],
        active_situation_penalty: v[6],
        threat_player_penalty: v[7],
        threat_system_penalty: v[8],
        death_penalty: v[9],
        gunner_base_reward: v[10],
        gunner_per_ammo: v[11],
        gunner_working_bonus: v[12],
        gunner_distance_factor: v[13],
        firefighter_base_reward: v[14],
        firefighter_distance_factor: v[15],
        baker_base_reward: v[16],
        baker_distance_factor: v[17],
        backtracking_penalty: v[18],
    }
}

fn mutate(rng: &mut impl Rng, genome: &mut Vec<f64>) {
    let idx = rng.gen_range(0..genome.len());
    // Mutate by +/- 20% or random small noise
    if rng.gen_bool(0.5) {
        genome[idx] *= rng.gen_range(0.8..1.2);
    } else {
        genome[idx] += rng.gen_range(-10.0..10.0);
    }
    // Ensure non-negative? Some penalties might be negative in usage, but here they are positive magnitudes.
    // The scoring function subtracts penalties. So magnitudes should be positive.
    if genome[idx] < 0.0 {
        genome[idx] = 0.0;
    }
}

fn evaluate(weights: &ScoringWeights, seeds: &[u64]) -> f64 {
    let total_score: f64 = seeds
        .par_iter()
        .map(|&seed| {
            let config = SearchConfig {
                players: 6,
                seed,
                width: 20, // Small width for speed
                steps: 40, // Short horizon
                time_limit: 5,
                verbose: false,
            };

            if let Some(sol) = beam_search(&config, weights) {
                let mut fitness = 0.0;

                // 1. Victory
                if sol.state.phase == GamePhase::Victory {
                    fitness += 5000.0;
                    // Faster victory bonus
                    fitness += (100 - sol.state.turn_count) as f64 * 100.0;
                }

                // 2. Hull Health
                fitness += sol.state.hull_integrity as f64 * 50.0;

                // 3. Boss Damage (Max - Current)
                fitness += (sol.state.enemy.max_hp - sol.state.enemy.hp) as f64 * 100.0;

                // 4. Survival Duration (if not won)
                if sol.state.phase != GamePhase::Victory {
                    fitness += sol.state.turn_count as f64 * 10.0;
                }

                fitness
            } else {
                0.0 // Died immediately or failed
            }
        })
        .sum();

    total_score / (seeds.len() as f64)
}

fn main() {
    let args = Args::parse();
    let seeds: Vec<u64> = args
        .seeds
        .split(',')
        .map(|s| s.trim().parse().unwrap())
        .collect();

    println!(
        "🧬 Starting Evolution: Gens={}, Pop={}, Seeds={:?}",
        args.generations, args.population, seeds
    );

    let mut rng = rand::thread_rng();

    // Initialize Population
    // Include default weights + variations
    let default_genome = weights_to_vec(&ScoringWeights::default());
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
                let w = vec_to_weights(genome);
                let score = evaluate(&w, &seeds);
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
    let best_weights = vec_to_weights(best_genome);
    println!("\n🏆 Best Weights Found:");
    println!("{:#?}", best_weights);
}
