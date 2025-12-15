use clap::{Parser, ValueEnum};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use sint_solver::optimization::{
    Checkpoint, OptimizationStatus, OptimizerConfig, OptimizerMessage, SeedResult, Strategy,
    Target, run_ga, run_spsa,
};
use sint_solver::scoring::beam::BeamScoringWeights;
use sint_solver::scoring::rhea::RheaScoringWeights;
use sint_solver::search::SearchProgress;
use std::{
    io,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use sysinfo::{Pid, ProcessesToUpdate, System};

use sint_solver::search::config::{BeamConfig, RHEAConfigParams};

const GB: f64 = 1024.0 * 1024.0 * 1024.0;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Optimization Strategy
    #[arg(long, value_enum, default_value_t = ArgStrategy::GA)]
    strategy: ArgStrategy,

    /// Target Search Algorithm
    #[arg(long, value_enum, default_value_t = ArgTarget::Beam)]
    target: ArgTarget,

    /// Generations / Iterations (Optimizer)
    #[arg(short, long, default_value_t = 20)]
    generations: usize,

    /// Population Size (Optimizer GA)
    #[arg(short, long, default_value_t = 20)]
    population: usize,

    /// Seeds to evaluate (comma separated)
    #[arg(long, default_value = "12345")]
    seeds: String,

    /// Checkpoint file path (loads if exists, saves periodically)
    #[arg(long)]
    checkpoint: Option<String>,

    /// TUI Mode
    #[arg(long, default_value_t = true)]
    tui: bool,

    #[command(flatten)]
    beam: BeamConfig,

    #[command(flatten)]
    rhea: RHEAConfigParams,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ArgStrategy {
    GA,
    Spsa,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ArgTarget {
    Beam,
    Rhea,
}

impl From<ArgStrategy> for Strategy {
    fn from(s: ArgStrategy) -> Self {
        match s {
            ArgStrategy::GA => Strategy::GA,
            ArgStrategy::Spsa => Strategy::Spsa,
        }
    }
}

impl From<ArgTarget> for Target {
    fn from(t: ArgTarget) -> Self {
        match t {
            ArgTarget::Beam => Target::Beam,
            ArgTarget::Rhea => Target::Rhea,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let seeds: Vec<u64> = args
        .seeds
        .split(',')
        .map(|s| s.trim().parse().unwrap())
        .collect();

    let config = OptimizerConfig {
        strategy: args.strategy.into(),
        target: args.target.into(),
        generations: args.generations,
        population: args.population,
        seeds: seeds.clone(),
        beam_width: args.beam.beam_width,
        rhea_horizon: args.rhea.rhea_horizon,
        rhea_generations: args.rhea.rhea_generations,
        rhea_population: args.rhea.rhea_population,
    };

    let checkpoint = if let Some(path) = &args.checkpoint {
        if std::path::Path::new(path).exists() {
            println!("🔄 Loading checkpoint from {}...", path);
            Some(Checkpoint::load(path)?)
        } else {
            None
        }
    } else {
        None
    };

    if args.tui {
        run_tui(config, checkpoint, args.checkpoint)
    } else {
        run_cli(config, checkpoint, args.checkpoint);
        Ok(())
    }
}

fn run_cli(
    config: OptimizerConfig,
    initial_checkpoint: Option<Checkpoint>,
    checkpoint_path: Option<String>,
) {
    println!(
        "🧬 Starting Optimization ({} -> {}): Gens={}, Pop={}, Seeds={:?}",
        match config.strategy {
            Strategy::GA => "GA",
            Strategy::Spsa => "SPSA",
        },
        match config.target {
            Target::Beam => "Beam",
            Target::Rhea => "Rhea",
        },
        config.generations,
        config.population,
        config.seeds
    );

    let (tx, rx) = mpsc::channel();
    let thread_config = config.clone();

    let mut current_generation = initial_checkpoint
        .as_ref()
        .map(|c| c.generation)
        .unwrap_or(0);
    let mut current_population = initial_checkpoint
        .as_ref()
        .map(|c| c.population.clone())
        .unwrap_or_default(); // Will be initialized in run_ga if empty
    let mut current_seed_results = initial_checkpoint
        .as_ref()
        .map(|c| c.seed_results.clone())
        .unwrap_or_default();
    let mut history = initial_checkpoint
        .as_ref()
        .map(|c| c.history.clone())
        .unwrap_or_default();

    thread::spawn(move || match thread_config.strategy {
        Strategy::GA => run_ga(&thread_config, tx, initial_checkpoint),
        Strategy::Spsa => run_spsa(&thread_config, tx, initial_checkpoint),
    });

    while let Ok(msg) = rx.recv() {
        match msg {
            OptimizerMessage::IndividualStarting { index, genome, .. } => {
                if index >= current_population.len() {
                    current_population.resize(index + 1, genome);
                } else {
                    current_population[index] = genome;
                }
            }
            OptimizerMessage::IndividualUpdate { .. } => {
                // Ignore live updates in CLI for now
            }
            OptimizerMessage::IndividualDone { .. } => {
                use std::io::Write;
                print!(".");
                io::stdout().flush().unwrap();
            }
            OptimizerMessage::SeedDone {
                index,
                seed_idx,
                metrics,
                ..
            } => {
                if let Some(m) = metrics {
                    current_seed_results.push(SeedResult {
                        ind_idx: index,
                        seed_idx,
                        metrics: m,
                    });

                    // Save checkpoint on every seed completion in CLI
                    if let Some(path) = &checkpoint_path {
                        let population_to_save = if config.strategy == Strategy::GA {
                            current_population.clone()
                        } else {
                            // SPSA: [current_theta, best_theta]
                            if current_population.is_empty() {
                                Vec::new()
                            } else {
                                vec![
                                    current_population[0].clone(),
                                    history
                                        .last()
                                        .map(|h| h.best_genome.clone())
                                        .unwrap_or_else(|| current_population[0].clone()),
                                ]
                            }
                        };

                        let ckpt = Checkpoint {
                            config: config.clone(),
                            generation: current_generation,
                            population: population_to_save,
                            seed_results: current_seed_results.clone(),
                            history: history.clone(),
                        };
                        let _ = ckpt.save(path);
                    }
                }
            }
            OptimizerMessage::GenerationDone(status) => {
                current_generation = status.generation + 1;
                current_seed_results.clear();
                history.push(*status.clone());

                // For SPSA, we store [current_theta, best_theta] in the population field
                let population_to_save = if config.strategy == Strategy::GA {
                    current_population.clone()
                } else {
                    // SPSA: [current_theta, best_theta]
                    if current_population.is_empty() {
                        vec![status.best_genome.clone(), status.best_genome.clone()]
                    } else {
                        vec![current_population[0].clone(), status.best_genome.clone()]
                    }
                };

                // Save checkpoint at end of generation
                if let Some(path) = &checkpoint_path {
                    let ckpt = Checkpoint {
                        config: config.clone(),
                        generation: current_generation,
                        population: population_to_save,
                        seed_results: Vec::new(),
                        history: history.clone(),
                    };
                    let _ = ckpt.save(path);
                }

                println!(); // Newline after dots
                println!(
                    "Gen {}: Best Score {:.2} | Avg Score {:.2}",
                    status.generation, status.best_score, status.avg_score
                );
                println!(
                    "  Wins: {} | Losses: {} | Timeouts: {} | Panics: {}",
                    status.best_metrics.wins,
                    status.best_metrics.losses,
                    status.best_metrics.timeouts,
                    status.best_metrics.panics
                );

                if status.generation == config.generations - 1 {
                    println!("\n🏆 Best Weights Found:");
                    if let Some(w) = status.current_weights_beam {
                        println!("{:#?}", w);
                    }
                    if let Some(w) = status.current_weights_rhea {
                        println!("{:#?}", w);
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct IndividualStatus {
    seed_statuses: Vec<u8>, // 0: Pending, 1: Running, 2: Win, 3: Loss, 4: Panic
    seed_histories: Vec<Vec<f32>>,
    current_progress: Vec<Option<SearchProgress>>,
    action_histories: Vec<Vec<String>>,
}

struct App {
    config: OptimizerConfig,
    history: Vec<OptimizationStatus>,
    population_status: Vec<IndividualStatus>,
    population_genomes: Vec<Vec<f64>>,
    done: bool,
    sys: System,
    pid: Pid,
    last_sys_update: Instant,
}

impl App {
    fn new(config: OptimizerConfig) -> App {
        let population_status = vec![
            IndividualStatus {
                seed_statuses: vec![0; config.seeds.len()],
                seed_histories: vec![Vec::new(); config.seeds.len()],
                current_progress: vec![None; config.seeds.len()],
                action_histories: vec![Vec::new(); config.seeds.len()],
            };
            config.population
        ];
        let mut sys = System::new_all();
        sys.refresh_all();
        let pid = sysinfo::get_current_pid().expect("Failed to get PID");

        App {
            config: config.clone(),
            history: Vec::new(),
            population_status,
            population_genomes: vec![Vec::new(); config.population],
            done: false,
            sys,
            pid,
            last_sys_update: Instant::now(),
        }
    }

    fn on_tick(&mut self) {
        if self.last_sys_update.elapsed() >= Duration::from_secs(2) {
            self.sys.refresh_cpu_all();
            self.sys.refresh_memory();
            self.sys
                .refresh_processes(ProcessesToUpdate::Some(&[self.pid]), true);
            self.last_sys_update = Instant::now();
        }
    }
}

fn run_tui(
    config: OptimizerConfig,
    initial_checkpoint: Option<Checkpoint>,
    checkpoint_path: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // App State
    let mut app = App::new(config.clone());

    // Apply checkpoint to app state if available
    if let Some(ckpt) = &initial_checkpoint {
        app.population_genomes = ckpt.population.clone();
        app.history = ckpt.history.clone();
        for res in &ckpt.seed_results {
            if res.ind_idx < app.population_status.len() {
                let ind = &mut app.population_status[res.ind_idx];
                if res.seed_idx < ind.seed_statuses.len() {
                    ind.seed_statuses[res.seed_idx] = res.metrics.get_status();
                }
            }
        }
    }

    // Local state for checkpointing
    let mut current_generation = initial_checkpoint
        .as_ref()
        .map(|c| c.generation)
        .unwrap_or(0);
    let mut current_seed_results = initial_checkpoint
        .as_ref()
        .map(|c| c.seed_results.clone())
        .unwrap_or_default();

    // Spawn Optimization Thread
    let (tx, rx) = mpsc::channel();
    let thread_config = config.clone();

    thread::spawn(move || match thread_config.strategy {
        Strategy::GA => run_ga(&thread_config, tx, initial_checkpoint),
        Strategy::Spsa => run_spsa(&thread_config, tx, initial_checkpoint),
    });

    // Run Loop
    let tick_rate = Duration::from_millis(100); // Throttle UI to 10Hz
    let mut last_tick = Instant::now();

    loop {
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)?
            && let Event::Key(key) = event::read()?
            && let KeyCode::Char('q') = key.code
        {
            break;
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            terminal.draw(|f| ui(f, &app))?;
            last_tick = Instant::now();
        }

        // Check for updates (process all available messages before next tick)
        while let Ok(msg) = rx.try_recv() {
            match msg {
                OptimizerMessage::IndividualStarting { index, genome, .. } => {
                    if index < app.population_genomes.len() {
                        app.population_genomes[index] = genome;
                    }
                }
                OptimizerMessage::IndividualUpdate {
                    index,
                    seed_idx,
                    progress,
                    score_history,
                    ..
                } => {
                    if index < app.population_status.len() {
                        let ind = &mut app.population_status[index];
                        if seed_idx < ind.seed_statuses.len() {
                            ind.seed_statuses[seed_idx] = 1; // Running
                            ind.seed_histories[seed_idx] = score_history;

                            // Update Action Ticker
                            if let Some((pid, act)) = &progress.node.last_action {
                                let action_str = format!(
                                    "[R{}] {} -> {:?}",
                                    progress.node.state.turn_count, pid, act
                                );
                                let history = &mut ind.action_histories[seed_idx];
                                if history.last() != Some(&action_str) {
                                    history.push(action_str);
                                    if history.len() > 20 {
                                        history.remove(0);
                                    }
                                }
                            }

                            ind.current_progress[seed_idx] = Some(progress);
                        }
                    }
                }
                OptimizerMessage::SeedDone {
                    index,
                    seed_idx,
                    status,
                    metrics,
                    ..
                } => {
                    if index < app.population_status.len() {
                        let ind = &mut app.population_status[index];
                        if seed_idx < ind.seed_statuses.len() {
                            ind.seed_statuses[seed_idx] = status;

                            if let Some(m) = metrics {
                                current_seed_results.push(SeedResult {
                                    ind_idx: index,
                                    seed_idx,
                                    metrics: m,
                                });

                                // Save checkpoint on every seed completion
                                if let Some(path) = &checkpoint_path {
                                    let ckpt = Checkpoint {
                                        config: app.config.clone(),
                                        generation: current_generation,
                                        population: app.population_genomes.clone(),
                                        seed_results: current_seed_results.clone(),
                                        history: app.history.clone(),
                                    };
                                    let _ = ckpt.save(path);
                                }
                            }
                        }
                    }
                }
                OptimizerMessage::IndividualDone { index, genome, .. } => {
                    if index < app.population_status.len() {
                        app.population_genomes[index] = genome;
                    }
                }
                OptimizerMessage::GenerationDone(status) => {
                    current_generation = status.generation + 1;
                    current_seed_results.clear();

                    if status.generation == app.config.generations - 1 {
                        app.done = true;
                    }
                    app.history.push(*status.clone());

                    // Save checkpoint at end of generation
                    if let Some(path) = &checkpoint_path {
                        let population_to_save = if app.config.strategy == Strategy::GA {
                            app.population_genomes.clone()
                        } else {
                            // SPSA: [current_theta, best_theta]
                            if app.population_genomes.is_empty() {
                                vec![status.best_genome.clone(), status.best_genome.clone()]
                            } else {
                                vec![
                                    app.population_genomes[0].clone(),
                                    status.best_genome.clone(),
                                ]
                            }
                        };

                        let ckpt = Checkpoint {
                            config: app.config.clone(),
                            generation: current_generation,
                            population: population_to_save,
                            seed_results: Vec::new(),
                            history: app.history.clone(),
                        };
                        let _ = ckpt.save(path);
                    }

                    // Reset status for next gen
                    for ind in &mut app.population_status {
                        for s in &mut ind.seed_statuses {
                            *s = 0;
                        }
                        for h in &mut ind.seed_histories {
                            h.clear();
                        }
                        for p in &mut ind.current_progress {
                            *p = None;
                        }
                        for a in &mut ind.action_histories {
                            a.clear();
                        }
                    }
                }
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Print best results found so far
    if let Some(last_status) = app.history.last() {
        println!("\n🏆 Best Weights Found (Gen {}):", last_status.generation);
        if let Some(w) = &last_status.current_weights_beam {
            println!("{:#?}", w);
        }
        if let Some(w) = &last_status.current_weights_rhea {
            println!("{:#?}", w);
        }
    } else {
        println!("\n⚠️ No generations completed.");
    }

    Ok(())
}

use ratatui::layout::Rect;
use sint_solver::tui::map::MapWidget;
use sint_solver::tui::optimization::{
    ActionTicker, GenerationSummary, GenomeMosaic, LiveStats, OptimizationHeader, ScoreBreakdown,
    ScoreRibbon, SeedGauntlet,
};

fn ui(f: &mut Frame, app: &App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),  // Header (Back to 4, using horizontal space)
            Constraint::Min(0),     // Main Content
            Constraint::Length(22), // Bottom Telemetry
        ])
        .split(area);

    // 1. Header
    let strategy_str = match app.config.strategy {
        Strategy::GA => "Genetic Algorithm (Crowding)",
        Strategy::Spsa => "SPSA",
    };
    let target_str = match app.config.target {
        Target::Beam => format!("Beam Search (Width {})", app.config.beam_width),
        Target::Rhea => format!(
            "RHEA (H{}, G{}, P{})",
            app.config.rhea_horizon, app.config.rhea_generations, app.config.rhea_population
        ),
    };
    let status_str = if app.done {
        "DONE (q to quit)"
    } else {
        "RUNNING (q to quit)"
    };

    // Calculate Progress
    let mut games_done = 0;
    let mut games_running = 0;
    let mut inds_done = 0;

    for ind in &app.population_status {
        let mut ind_seeds_done = 0;
        for &s in &ind.seed_statuses {
            if s >= 2 {
                games_done += 1;
                ind_seeds_done += 1;
            } else if s == 1 {
                games_running += 1;
            }
        }
        if ind_seeds_done == ind.seed_statuses.len() {
            inds_done += 1;
        }
    }

    let total_games = app.config.population * app.config.seeds.len();
    let games_pending = total_games.saturating_sub(games_done + games_running);

    // System Stats
    let cpu_usage = app.sys.global_cpu_usage();
    let mem_used = app.sys.used_memory() as f64 / GB;
    let mem_total = app.sys.total_memory() as f64 / GB;
    let proc_mem = app
        .sys
        .process(app.pid)
        .map(|p| p.memory() as f64 / GB)
        .unwrap_or(0.0);

    f.render_widget(
        OptimizationHeader {
            strategy: strategy_str,
            target: &target_str,
            generation: app.history.len(),
            max_generations: app.config.generations,
            population: app.config.population,
            status: status_str,
            games_done,
            total_games,
            games_pending,
            games_running,
            inds_done,
            cpu_usage,
            mem_proc: proc_mem,
            mem_used,
            mem_total,
        },
        chunks[0],
    );

    // 2. Main Content
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(22), // Top Part: Mosaic, Gauntlet, Summary
            Constraint::Min(0),     // Middle Part: Score Ribbons
        ])
        .split(chunks[1]);

    // A. Top Part (Horizontal)
    let top_part_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(120), // Genome Mosaic (Enough for 109 params)
            Constraint::Length(60),  // Seed Gauntlet
            Constraint::Min(0),      // Summary
        ])
        .split(main_chunks[0]);

    // A1. Genome Mosaic
    let param_names = match app.config.target {
        Target::Beam => sint_solver::optimization::get_param_names::<BeamScoringWeights>(),
        Target::Rhea => sint_solver::optimization::get_param_names::<RheaScoringWeights>(),
    };

    let mosaic_title = Line::from(vec![
        Span::raw("Genome Mosaic (DNA: "),
        Span::styled("Blue", Style::default().fg(Color::Blue)),
        Span::raw("<1.0<"),
        Span::styled("Red", Style::default().fg(Color::Red)),
        Span::raw(")"),
    ]);

    f.render_widget(
        GenomeMosaic {
            population: &app.population_genomes,
            param_names: &param_names,
            block: None,
        }
        .block(Block::default().title(mosaic_title).borders(Borders::ALL)),
        top_part_chunks[0],
    );

    // A2. Seed Gauntlet
    let gauntlet_data: Vec<Vec<u8>> = app
        .population_status
        .iter()
        .map(|ind| ind.seed_statuses.clone())
        .collect();

    let gauntlet_title = Line::from(vec![
        Span::raw("Seed Gauntlet ("),
        Span::styled("⠶", Style::default().fg(Color::Blue)),
        Span::raw(":Run "),
        Span::styled("█", Style::default().fg(Color::Green)),
        Span::raw(":Win "),
        Span::styled("█", Style::default().fg(Color::Red)),
        Span::raw(":Loss "),
        Span::styled("!", Style::default().fg(Color::Magenta)),
        Span::raw(":Err "),
        Span::styled("█", Style::default().fg(Color::Yellow)),
        Span::raw(":Time)"),
    ]);

    f.render_widget(
        SeedGauntlet {
            status: &gauntlet_data,
            seed_labels: &app.config.seeds,
            block: None,
        }
        .block(Block::default().title(gauntlet_title).borders(Borders::ALL)),
        top_part_chunks[1],
    );

    // A3. Summary
    let summary_widget = if let Some(last) = app.history.last() {
        GenerationSummary {
            wins: last.best_metrics.wins,
            losses: last.best_metrics.losses,
            timeouts: last.best_metrics.timeouts,
            best_score: last.best_score,
            avg_score: last.avg_score,
            is_loading: false,
            block: None,
        }
    } else {
        GenerationSummary {
            wins: 0,
            losses: 0,
            timeouts: 0,
            best_score: 0.0,
            avg_score: 0.0,
            is_loading: true,
            block: None,
        }
    };

    f.render_widget(
        summary_widget.block(
            Block::default()
                .title("Generation Summary")
                .borders(Borders::ALL),
        ),
        top_part_chunks[2],
    );

    // B. Score Ribbons (Middle part, full width)
    let mut ribbon_data: Vec<(usize, usize, f32, Vec<f32>)> = Vec::new();
    for (i, status) in app.population_status.iter().enumerate() {
        let mut running_any = false;
        for (seed_idx, &s) in status.seed_statuses.iter().enumerate() {
            if s == 1 {
                let history = &status.seed_histories[seed_idx];
                let avg_health = history.iter().sum::<f32>() / history.len().max(1) as f32;
                ribbon_data.push((i, seed_idx, avg_health, history.clone()));
                running_any = true;
            }
        }
        if !running_any {
            // If none running, find the one with longest history
            let best_idx = status
                .seed_histories
                .iter()
                .enumerate()
                .max_by_key(|(_, h)| h.len())
                .map(|(idx, _)| idx)
                .unwrap_or(0);
            let history = &status.seed_histories[best_idx];
            let avg_health = history.iter().sum::<f32>() / history.len().max(1) as f32;
            ribbon_data.push((i, best_idx, avg_health, history.clone()));
        }
    }

    ribbon_data.sort_by(|a, b| {
        let a_running = app.population_status[a.0].seed_statuses[a.1] == 1;
        let b_running = app.population_status[b.0].seed_statuses[b.1] == 1;
        if a_running != b_running {
            if a_running {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        } else {
            b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal)
        }
    });

    let ribbon_title = Line::from(vec![
        Span::raw("Top Performances (Health: "),
        Span::styled("Red", Style::default().fg(Color::Red)),
        Span::raw("->"),
        Span::styled("Green", Style::default().fg(Color::Green)),
        Span::raw(")"),
    ]);
    let ribbon_block = Block::default().title(ribbon_title).borders(Borders::ALL);
    let ribbon_inner = ribbon_block.inner(main_chunks[1]);
    f.render_widget(ribbon_block, main_chunks[1]);

    for (i, (ind_idx, seed_idx, _score, history)) in ribbon_data
        .iter()
        .take(ribbon_inner.height as usize)
        .enumerate()
    {
        f.render_widget(
            ScoreRibbon {
                data: history,
                label: &format!("C{:02} S{:02}", ind_idx, seed_idx),
                max_rounds: 200,
            },
            Rect {
                x: ribbon_inner.x,
                y: ribbon_inner.y + i as u16,
                width: ribbon_inner.width,
                height: 1,
            },
        );
    }

    // 3. Bottom Telemetry (Best Individual Live View)
    // We pick the best individual's active or longest history seed.
    let best_info = ribbon_data.first();
    let best_progress = best_info.and_then(|(idx, seed_idx, _, _)| {
        app.population_status[*idx].current_progress[*seed_idx].as_ref()
    });

    if let Some(prog) = best_progress {
        let (ind_idx, seed_idx, _, _) = best_info.unwrap();
        let ind_label = format!("(C{:02} S{:02})", ind_idx, seed_idx);
        let tele_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(85), // Map (Increased from 65)
                Constraint::Length(30), // Stats
                Constraint::Length(40), // Score Breakdown
                Constraint::Min(0),     // Action Log
            ])
            .split(chunks[2]);

        // Map
        f.render_widget(
            MapWidget {
                state: Some(&prog.node.state),
                block: None,
            }
            .block(
                Block::default()
                    .title(format!("Ship Map {}", ind_label))
                    .borders(Borders::ALL),
            ),
            tele_chunks[0],
        );

        // Stats
        f.render_widget(
            LiveStats {
                state: &prog.node.state,
                score: prog.node.score.total,
                label: &ind_label,
            },
            tele_chunks[1],
        );

        // Score Breakdown
        f.render_widget(
            ScoreBreakdown {
                score: &prog.node.score,
                label: &ind_label,
            },
            tele_chunks[2],
        );

        // Action Ticker (Full History)
        f.render_widget(
            ActionTicker {
                history: &app.population_status[*ind_idx].action_histories[*seed_idx],
                label: &ind_label,
            },
            tele_chunks[3],
        );
    } else {
        f.render_widget(
            Paragraph::new("Waiting for evaluation data...")
                .block(Block::default().borders(Borders::ALL)),
            chunks[2],
        );
    }
}
