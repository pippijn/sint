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
    text::{Line, Span},
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

use sint_solver::search::config::{BeamConfig, RHEAConfigParams};

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
            OptimizerMessage::IndividualStarting { .. } => {
                // Ignore in CLI
            }
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

        if crossterm::event::poll(timeout)?
            && let Event::Key(key) = event::read()?
            && let KeyCode::Char('q') = key.code
        {
            break;
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }

        // Check for updates
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
            Constraint::Length(4),  // Header (Increased from 3)
            Constraint::Min(0),     // Main Content
            Constraint::Length(22), // Bottom Telemetry (Increased from 15)
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

    let header_text = format!(
        "SINT OPTIMIZER | Strat: {} | Target: {} | Gen: {}/{} | Pop: {} | {}\n\
         WORKLOAD: Games: {}/{} ({} Pending, {} Running) | Genomes: {}/{} Completed",
        strategy_str,
        target_str,
        app.history.len(),
        app.config.generations,
        app.config.population,
        status_str,
        games_done,
        total_games,
        games_pending,
        games_running,
        inds_done,
        app.config.population
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
            top_part_chunks[2],
        );
    }

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
        let history = &app.population_status[*ind_idx].action_histories[*seed_idx];
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
