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
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Row, Table},
};
use sint_solver::optimization::{
    OptimizationStatus, OptimizerConfig, OptimizerMessage, Strategy, Target, run_ga, run_spsa,
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

    /// Enable TUI mode
    #[arg(long)]
    tui: bool,

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
        rhea_horizon: args.rhea_horizon,
        rhea_generations: args.rhea_generations,
        rhea_population: args.rhea_population,
    };

    if args.tui {
        run_tui(config)
    } else {
        run_cli(config);
        Ok(())
    }
}

fn run_cli(config: OptimizerConfig) {
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

    thread::spawn(move || match thread_config.strategy {
        Strategy::GA => run_ga(&thread_config, tx),
        Strategy::Spsa => run_spsa(&thread_config, tx),
    });

    while let Ok(msg) = rx.recv() {
        match msg {
            OptimizerMessage::IndividualUpdate { .. } => {
                // Ignore live updates in CLI for now
            }
            OptimizerMessage::IndividualDone { .. } => {
                use std::io::Write;
                print!(".");
                io::stdout().flush().unwrap();
            }
            OptimizerMessage::SeedDone { .. } => {
                // Ignore per-seed updates in CLI
            }
            OptimizerMessage::GenerationDone(status) => {
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
        App {
            config: config.clone(),
            history: Vec::new(),
            population_status,
            population_genomes: vec![Vec::new(); config.population],
            done: false,
        }
    }

    fn on_tick(&mut self) {}
}

fn run_tui(config: OptimizerConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // App State
    let mut app = App::new(config.clone());

    // Spawn Optimization Thread
    let (tx, rx) = mpsc::channel();
    let thread_config = config.clone();

    thread::spawn(move || match thread_config.strategy {
        Strategy::GA => run_ga(&thread_config, tx),
        Strategy::Spsa => run_spsa(&thread_config, tx),
    });

    // Run Loop
    let tick_rate = Duration::from_millis(50); // Faster tick for smooth UI updates
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }

        // Check for updates
        while let Ok(msg) = rx.try_recv() {
            match msg {
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
                                    if history.len() > 15 {
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
                    ..
                } => {
                    if index < app.population_status.len() {
                        let ind = &mut app.population_status[index];
                        if seed_idx < ind.seed_statuses.len() {
                            ind.seed_statuses[seed_idx] = status;
                        }
                    }
                }
                OptimizerMessage::IndividualDone { index, genome, .. } => {
                    if index < app.population_status.len() {
                        app.population_genomes[index] = genome;
                    }
                }
                OptimizerMessage::GenerationDone(status) => {
                    if status.generation == app.config.generations - 1 {
                        app.done = true;
                    }
                    app.history.push(*status);
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
use sint_solver::tui::optimization::{GenomeMosaic, ScoreRibbon, SeedGauntlet};

fn ui(f: &mut Frame, app: &App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Main Content
            Constraint::Length(15), // Bottom Telemetry
        ])
        .split(area);

    // 1. Header
    let strategy_str = match app.config.strategy {
        Strategy::GA => "Genetic Algorithm (Crowding)",
        Strategy::Spsa => "SPSA",
    };
    let target_str = match app.config.target {
        Target::Beam => "Beam Search (Width 100)",
        Target::Rhea => "RHEA (H10, G50, P20)",
    };
    let status_str = if app.done {
        "DONE (q to quit)"
    } else {
        "RUNNING (q to quit)"
    };

    let header_text = format!(
        "SINT OPTIMIZER | Strat: {} | Target: {} | Gen: {}/{} | Pop: {} | {}",
        strategy_str,
        target_str,
        app.history.len(),
        app.config.generations,
        app.config.population,
        status_str
    );

    f.render_widget(
        Paragraph::new(header_text)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL)),
        chunks[0],
    );

    // 2. Main Content (Mosaic, Ribbons, Gauntlet)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(45), // Genome Mosaic
            Constraint::Min(0),     // Score Ribbons
            Constraint::Length(75), // Seed Gauntlet + Stats
        ])
        .split(chunks[1]);

    // A. Genome Mosaic
    let param_names = match app.config.target {
        Target::Beam => sint_solver::optimization::get_param_names::<BeamScoringWeights>(),
        Target::Rhea => sint_solver::optimization::get_param_names::<RheaScoringWeights>(),
    };

    f.render_widget(
        GenomeMosaic {
            population: &app.population_genomes,
            param_names: &param_names,
            block: None,
        }
        .block(
            Block::default()
                .title("Genome Mosaic (DNA)")
                .borders(Borders::ALL),
        ),
        main_chunks[0],
    );

    // B. Score Ribbons
    // Find top 20 individuals based on average health score
    let mut ribbon_data: Vec<(usize, f32, Vec<f32>)> = app
        .population_status
        .iter()
        .enumerate()
        .map(|(i, status)| {
            // Average of first seed's history for visualization
            let history = status.seed_histories[0].clone();
            let avg_health = history.iter().sum::<f32>() / history.len().max(1) as f32;
            (i, avg_health, history)
        })
        .collect();

    ribbon_data.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let ribbon_list_area = main_chunks[1];
    let ribbon_block = Block::default()
        .title("Top Performances (Health over Rounds)")
        .borders(Borders::ALL);
    let ribbon_inner = ribbon_block.inner(ribbon_list_area);
    f.render_widget(ribbon_block, ribbon_list_area);

    for (i, (ind_idx, _score, history)) in ribbon_data
        .iter()
        .take(ribbon_inner.height as usize)
        .enumerate()
    {
        f.render_widget(
            ScoreRibbon {
                data: history,
                label: &format!("C{:02}", ind_idx),
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

    // C. Seed Gauntlet + Best Result Table
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60), // Gauntlet
            Constraint::Percentage(40), // Outcome summary
        ])
        .split(main_chunks[2]);

    let gauntlet_data: Vec<Vec<u8>> = app
        .population_status
        .iter()
        .map(|ind| ind.seed_statuses.clone())
        .collect();

    f.render_widget(
        SeedGauntlet {
            status: &gauntlet_data,
            seed_labels: &app.config.seeds,
            block: None,
        }
        .block(
            Block::default()
                .title("Seed Gauntlet (⠶:Run █:Win █:Loss !:Err █:Time)")
                .borders(Borders::ALL),
        ),
        right_chunks[0],
    );

    if let Some(last) = app.history.last() {
        let m = &last.best_metrics;
        let wins = m.wins.to_string();
        let losses = m.losses.to_string();
        let timeouts = m.timeouts.to_string();
        let best_score = format!("{:.1}", last.best_score);
        let avg_score = format!("{:.1}", last.avg_score);

        let rows = vec![
            Row::new(vec!["Wins", wins.as_str()]).style(Style::default().fg(Color::Green)),
            Row::new(vec!["Losses", losses.as_str()]).style(Style::default().fg(Color::Red)),
            Row::new(vec!["Timeouts", timeouts.as_str()]).style(Style::default().fg(Color::Yellow)),
            Row::new(vec!["Best Score", best_score.as_str()]),
            Row::new(vec!["Avg Score", avg_score.as_str()]),
        ];
        f.render_widget(
            Table::new(rows, [Constraint::Length(15), Constraint::Min(0)]).block(
                Block::default()
                    .title("Generation Summary")
                    .borders(Borders::ALL),
            ),
            right_chunks[1],
        );
    }

    // 3. Bottom Telemetry (Best Individual Live View)
    // We pick the best individual's first seed that is currently running or done.
    let best_idx = ribbon_data.first().map(|(idx, _, _)| *idx);
    let best_ind = best_idx.and_then(|idx| app.population_status.get(idx));
    let best_progress = best_ind.and_then(|ind| ind.current_progress[0].as_ref());

    if let Some(prog) = best_progress {
        let ind_label = format!("(C{:02})", best_idx.unwrap());
        let tele_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(40), // Map
                Constraint::Length(30), // Stats
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
        let stats_text = format!(
            "Round: {}\nHull: {}/20\nBoss HP: {}/{}\nPhase: {:?}\nScore: {:.1}",
            prog.node.state.turn_count,
            prog.node.state.hull_integrity,
            prog.node.state.enemy.hp,
            prog.node.state.enemy.max_hp,
            prog.node.state.phase,
            prog.node.score.total
        );
        f.render_widget(
            Paragraph::new(stats_text).block(
                Block::default()
                    .title(format!("Live Stats {}", ind_label))
                    .borders(Borders::ALL),
            ),
            tele_chunks[1],
        );

        // Action Ticker (Full History)
        let history = best_ind.map(|ind| &ind.action_histories[0]).unwrap();
        let ticker_text = history.join("\n");
        f.render_widget(
            Paragraph::new(ticker_text).block(
                Block::default()
                    .title(format!("Action Ticker {}", ind_label))
                    .borders(Borders::ALL),
            ),
            tele_chunks[2],
        );
    } else {
        f.render_widget(
            Paragraph::new("Waiting for evaluation data...")
                .block(Block::default().borders(Borders::ALL)),
            chunks[2],
        );
    }
}
