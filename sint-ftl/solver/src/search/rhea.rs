use crate::driver::GameDriver;
use crate::scoring::rhea::score_rhea;
use crate::search::{get_legal_actions, get_state_signature, SearchNode};
use rand::prelude::*;
use rand::rngs::StdRng;
use rayon::prelude::*;
use sint_core::logic::GameLogic;
use sint_core::types::{GameAction, GamePhase, GameState, PlayerId};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct RHEAConfig {
    pub players: usize,
    pub seed: u64,
    pub horizon: usize,
    pub generations: usize,
    pub population_size: usize,
    pub max_steps: usize,
    pub time_limit: u64,
    pub verbose: bool,
}

#[derive(Clone)]
struct Individual {
    actions: Vec<(PlayerId, GameAction)>,
    score: f64,
}

pub fn rhea_search(config: &RHEAConfig) -> Option<SearchNode> {
    let player_ids: Vec<String> = (0..config.players).map(|i| format!("P{}", i + 1)).collect();
    let initial_state = GameLogic::new_game(player_ids, config.seed);

    // Stabilize root
    let root_driver = GameDriver::new(initial_state.clone());
    let mut current_state = root_driver.state.clone();

    let start_time = Instant::now();
    let time_limit = Duration::from_secs(config.time_limit);

    // Construct the trajectory chain as we play
    let mut search_node_chain: Option<Arc<SearchNode>> = Some(Arc::new(SearchNode {
        state: current_state.clone(),
        parent: None,
        last_action: None,
        score: 0.0,
        signature: get_state_signature(&current_state),
    }));

    if config.verbose {
        println!(
            "🧬 Starting RHEA: Horizon={}, Gens={}, Pop={}",
            config.horizon, config.generations, config.population_size
        );
    }

    let mut population: Vec<Individual> = Vec::new();
    let mut steps_taken = 0;

    // Seed offset counter to ensure every generation/step gets unique seeds
    let mut seed_counter: u64 = 0;

    loop {
        // Game Loop
        if current_state.phase == GamePhase::GameOver || current_state.phase == GamePhase::Victory {
            break;
        }
        if start_time.elapsed() > time_limit {
            if config.verbose {
                println!("⏰ Time limit reached.");
            }
            break;
        }
        if steps_taken >= config.max_steps {
            if config.verbose {
                println!("🛑 Max steps reached.");
            }
            break;
        }
        steps_taken += 1;

        // 1. Initialize Population if empty (First turn)
        if population.is_empty() {
            population = (0..config.population_size)
                .into_par_iter()
                .enumerate()
                .map(|(i, _)| {
                    let mut rng = StdRng::seed_from_u64(config.seed + seed_counter + i as u64);
                    generate_random_individual(&current_state, config.horizon, &mut rng)
                })
                .collect();
            seed_counter += config.population_size as u64;
        }

        // 2. Evolve
        for gen in 0..config.generations {
            if start_time.elapsed() > time_limit {
                break;
            }

            let eval_seed_base = seed_counter;
            seed_counter += population.len() as u64;

            // Parallel Evaluation
            population.par_iter_mut().enumerate().for_each(|(i, ind)| {
                if ind.score == 0.0 {
                    let mut rng = StdRng::seed_from_u64(config.seed + eval_seed_base + i as u64);
                    ind.score = evaluate_individual(ind, &current_state, config, &mut rng);
                }
            });

            // Sort
            population.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

            // OPTIMIZATION: If this is the last generation or time is up,
            // do NOT produce offspring, as they won't be evaluated or used.
            if gen == config.generations - 1 || start_time.elapsed() > time_limit {
                break;
            }

            // Elitism
            let elite = population[0].clone();

            // Parallel Next Gen Creation
            let next_gen_seed_base = seed_counter;
            seed_counter += config.population_size as u64;

            let pop_ref = &population; // read-only reference

            let mut next_pop: Vec<Individual> = (0..config.population_size - 1) // -1 for elite
                .into_par_iter()
                .enumerate()
                .map(|(i, _)| {
                    let mut rng =
                        StdRng::seed_from_u64(config.seed + next_gen_seed_base + i as u64);
                    // Tournament
                    let p1 = &pop_ref[rng.gen_range(0..pop_ref.len())];
                    let p2 = &pop_ref[rng.gen_range(0..pop_ref.len())];
                    let parent = if p1.score > p2.score { p1 } else { p2 };

                    let mut child = parent.clone();
                    mutate(&mut child, config, &mut rng);
                    child.score = 0.0;
                    child
                })
                .collect();

            next_pop.push(elite);
            population = next_pop;
        }

        // 3. Select Best
        // Population is already sorted from the end of the loop (or initialization)
        let best_ind = &population[0];

        if config.verbose && (steps_taken % 10 == 0 || steps_taken == config.max_steps) {
            println!("RHEA Step {}: Best Score {:.1} | Round {} | Phase {:?} | Hull {} | Boss {} | Plan {}", 
                steps_taken,
                best_ind.score,
                current_state.turn_count,
                current_state.phase,
                current_state.hull_integrity,
                current_state.enemy.hp,
                best_ind.actions.len()
            );
        }

        if best_ind.actions.is_empty() {
            if config.verbose {
                println!("⚠️ RHEA found no valid actions. Stopping.");
            }
            break;
        }

        // Execute ONE step
        let (pid, act) = &best_ind.actions[0];
        let mut driver = GameDriver {
            state: current_state.clone(),
        };

        match driver.apply(pid, act.clone()) {
            Ok(_) => {
                let next_state = driver.state;
                let new_sn = Arc::new(SearchNode {
                    state: next_state.clone(),
                    parent: search_node_chain.clone(),
                    last_action: Some((pid.clone(), act.clone())),
                    score: best_ind.score, // Approximate score
                    signature: get_state_signature(&next_state),
                });
                search_node_chain = Some(new_sn);
                current_state = next_state;

                // 4. Seeding: Shift Population
                // Best individual becomes the seed.
                let mut seed_actions = best_ind.actions.clone();
                if !seed_actions.is_empty() {
                    seed_actions.remove(0); // Remove executed action
                }
                let seed_ind = Individual {
                    actions: seed_actions,
                    score: 0.0,
                };

                let mut new_pop = Vec::with_capacity(config.population_size);
                new_pop.push(seed_ind.clone()); // Elite (Shifted)

                // Diversity: 80% Mutants of Seed, 20% Fresh Randoms
                let num_mutants = (config.population_size as f64 * 0.8) as usize;

                // Fill with mutants
                let mutant_seed_base = seed_counter;
                seed_counter += num_mutants as u64;

                while new_pop.len() < 1 + num_mutants && new_pop.len() < config.population_size {
                    let mut mutant = seed_ind.clone();
                    let mut rng = StdRng::seed_from_u64(
                        config.seed + mutant_seed_base + new_pop.len() as u64,
                    );
                    mutate(&mut mutant, config, &mut rng);
                    mutant.score = 0.0;
                    new_pop.push(mutant);
                }

                // Fill remainder with random immigrants
                let immigrant_seed_base = seed_counter;
                seed_counter += config.population_size as u64;

                while new_pop.len() < config.population_size {
                    let mut rng = StdRng::seed_from_u64(
                        config.seed + immigrant_seed_base + new_pop.len() as u64,
                    );
                    new_pop.push(generate_random_individual(
                        &current_state,
                        config.horizon,
                        &mut rng,
                    ));
                }
                population = new_pop;
            }
            Err(e) => {
                if config.verbose {
                    println!("❌ RHEA selected invalid action: {:?} Error: {}", act, e);
                }
                break;
            }
        }
    }

    // Unwrap Arc
    search_node_chain.map(|arc| (*arc).clone())
}

fn generate_random_individual(state: &GameState, horizon: usize, rng: &mut StdRng) -> Individual {
    let mut actions = Vec::new();
    let mut sim_state = state.clone();

    for _ in 0..horizon {
        if sim_state.phase == GamePhase::GameOver || sim_state.phase == GamePhase::Victory {
            break;
        }
        let mut driver = GameDriver {
            state: sim_state.clone(),
        };
        let legal = get_legal_actions(&driver.state);
        if legal.is_empty() {
            break;
        }

        let (pid, act) = legal.choose(rng).unwrap();
        if driver.apply(pid, act.clone()).is_ok() {
            actions.push((pid.clone(), act.clone()));
            sim_state = driver.state;
        } else {
            break;
        }
    }
    Individual {
        actions,
        score: 0.0,
    }
}

fn evaluate_individual(
    ind: &mut Individual,
    start_state: &GameState,
    config: &RHEAConfig,
    rng: &mut impl Rng,
) -> f64 {
    let mut driver = GameDriver {
        state: start_state.clone(),
    };

    // Repair/Simulation
    let mut repaired_actions = Vec::new();

    for i in 0..ind.actions.len() {
        if driver.state.phase == GamePhase::GameOver || driver.state.phase == GamePhase::Victory {
            break;
        }

        let (pid, act) = &ind.actions[i];

        let mut effective_action = (pid.clone(), act.clone());
        let mut success = false;

        let legal = get_legal_actions(&driver.state);
        if legal.is_empty() {
            break;
        }

        // Check validity
        let is_valid = legal.iter().any(|(p, a)| *p == *pid && *a == *act);

        if is_valid {
            if driver.apply(pid, act.clone()).is_ok() {
                success = true;
            }
        }

        if !success {
            // Repair: Pick a random valid action
            let (new_pid, new_act) = legal.choose(rng).unwrap();
            if driver.apply(new_pid, new_act.clone()).is_ok() {
                effective_action = (new_pid.clone(), new_act.clone());
                success = true;
            }
        }

        if success {
            repaired_actions.push(effective_action);
        } else {
            break;
        }
    }

    // Write back repaired genome
    ind.actions = repaired_actions;

    // Regrow if short
    while ind.actions.len() < config.horizon {
        if driver.state.phase == GamePhase::GameOver || driver.state.phase == GamePhase::Victory {
            break;
        }
        let legal = get_legal_actions(&driver.state);
        if legal.is_empty() {
            break;
        }
        let (pid, act) = legal.choose(rng).unwrap();
        if driver.apply(pid, act.clone()).is_ok() {
            ind.actions.push((pid.clone(), act.clone()));
        } else {
            break;
        }
    }

    // Evaluate final state
    score_rhea(&driver.state)
}

fn mutate(ind: &mut Individual, _config: &RHEAConfig, rng: &mut impl Rng) {
    if ind.actions.is_empty() {
        return;
    }

    // Truncate mutation: Cut the tail and let evaluation regrow it.
    let cut_point = rng.gen_range(0..ind.actions.len());
    ind.actions.truncate(cut_point);
}
