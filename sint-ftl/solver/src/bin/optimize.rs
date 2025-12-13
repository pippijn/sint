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
    symbols,
    text::Span,
    widgets::{Axis, BarChart, Block, Borders, Chart, Dataset, GraphType, Paragraph, Row, Table},
};
use sint_solver::optimization::{
    EvaluationMetrics, OptimizationStatus, OptimizerConfig, OptimizerMessage, Strategy, Target,
    run_ga, run_spsa,
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
    #[arg(short, long, default_value_t = 40)]
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
enum ProgressState {
    Running(SearchProgress),
    Done(EvaluationMetrics),
}

struct App {
    config: OptimizerConfig,
    history: Vec<OptimizationStatus>,
    current_gen_progress: Vec<Option<ProgressState>>,
    done: bool,
}

impl App {
    fn new(config: OptimizerConfig) -> App {
        App {
            config: config.clone(),
            history: Vec::new(),
            current_gen_progress: vec![None; config.population],
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
    let tick_rate = Duration::from_millis(100); // Faster tick for smooth UI updates
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
                OptimizerMessage::IndividualUpdate {
                    index, progress, ..
                } => {
                    if index < app.current_gen_progress.len() {
                        app.current_gen_progress[index] = Some(ProgressState::Running(progress));
                    }
                }
                OptimizerMessage::IndividualDone { index, metrics, .. } => {
                    if index < app.current_gen_progress.len() {
                        app.current_gen_progress[index] = Some(ProgressState::Done(metrics));
                    }
                }
                OptimizerMessage::GenerationDone(status) => {
                    if status.generation == app.config.generations - 1 {
                        app.done = true;
                    }
                    app.history.push(*status);
                    // Reset progress for next gen
                    for p in &mut app.current_gen_progress {
                        *p = None;
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

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),      // Header
                Constraint::Percentage(40), // Chart
                Constraint::Length(10),     // Stats Bar + Population Grid
                Constraint::Min(0),         // Weights
            ]
            .as_ref(),
        )
        .split(f.area());

    // ... Header and Chart remain same ...
    // Header
    let strategy_str = match app.config.strategy {
        Strategy::GA => "Genetic Algorithm",
        Strategy::Spsa => "SPSA",
    };
    let target_str = match app.config.target {
        Target::Beam => "Beam Search",
        Target::Rhea => "RHEA",
    };

    let status_str = if app.done {
        "DONE - Press 'q' to quit"
    } else {
        "RUNNING - Press 'q' to quit"
    };

    let header_text = format!(
        "Optimizer Visualization | Strategy: {} | Target: {} | Gen: {}/{} | Pop: {} | {}",
        strategy_str,
        target_str,
        app.history.len(),
        app.config.generations,
        app.config.population,
        status_str
    );

    let header = Paragraph::new(header_text)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(header, chunks[0]);

    // Chart
    let best_data: Vec<(f64, f64)> = app
        .history
        .iter()
        .map(|s| (s.generation as f64, s.best_score))
        .collect();

    let avg_data: Vec<(f64, f64)> = app
        .history
        .iter()
        .map(|s| (s.generation as f64, s.avg_score))
        .collect();

    let datasets = vec![
        Dataset::default()
            .name("Best Score")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Green))
            .data(&best_data),
        Dataset::default()
            .name("Avg Score")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Yellow))
            .data(&avg_data),
    ];

    let x_max = app.config.generations as f64;
    // Find y range
    let y_min = app
        .history
        .iter()
        .map(|s| s.avg_score)
        .fold(f64::INFINITY, |a, b| a.min(b))
        .min(0.0);
    let y_max = app
        .history
        .iter()
        .map(|s| s.best_score)
        .fold(f64::NEG_INFINITY, |a, b| a.max(b))
        .max(100.0);

    // Add some padding
    let y_min = y_min - (y_max - y_min).abs() * 0.1;
    let y_max = y_max + (y_max - y_min).abs() * 0.1;

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title("Fitness History")
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .title("Generation")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, x_max])
                .labels(vec![
                    Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(
                        format!("{}", x_max),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                ]),
        )
        .y_axis(
            Axis::default()
                .title("Score")
                .style(Style::default().fg(Color::Gray))
                .bounds([y_min, y_max])
                .labels(vec![
                    Span::styled(
                        format!("{:.0}", y_min),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{:.0}", y_max),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                ]),
        );
    f.render_widget(chart, chunks[1]);

    // Split Stats Area: Left for History/Stats, Right for Current Gen Population
    let stats_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(chunks[2]);

    // Outcome Stats (Best Individual of Current Gen)
    if let Some(last_status) = app.history.last() {
        let m = &last_status.best_metrics;

        let wins = m.wins as u64;
        let losses = m.losses as u64;
        let timeouts = m.timeouts as u64;
        let panics = m.panics as u64;
        let total = wins + losses + timeouts + panics;

        let bars = vec![
            ("Wins", wins),
            ("Loss", losses),
            ("Time", timeouts),
            ("Fail", panics),
        ];

        let barchart = BarChart::default()
            .block(
                Block::default()
                    .title(format!("Outcomes (Best, N={})", total))
                    .borders(Borders::ALL),
            )
            .data(&bars)
            .bar_width(8)
            .bar_style(Style::default().fg(Color::Yellow))
            .value_style(Style::default().fg(Color::Black).bg(Color::Yellow));

        f.render_widget(barchart, stats_chunks[0]);
    } else {
        let block = Block::default().title("Outcomes").borders(Borders::ALL);
        f.render_widget(block, stats_chunks[0]);
    }

    // Population Visualization
    // Render a grid of blocks representing individuals.
    let mut pop_spans = Vec::new();
    for p in app.current_gen_progress.iter() {
        let (text, style) = match p {
            Some(ProgressState::Running(prog)) => {
                let color = if prog.node.state.hull_integrity <= 0 {
                    Color::Red
                } else if prog.is_done {
                    Color::Green
                } else {
                    Color::Blue
                };
                (
                    format!(
                        " R{:03} | H{:02} | S{:.0} ",
                        prog.step, prog.node.state.hull_integrity, prog.node.score.total
                    ),
                    Style::default().bg(color).fg(Color::White),
                )
            }
            Some(ProgressState::Done(m)) => {
                let color = if m.wins > 0 {
                    Color::Green
                } else if m.losses > 0 {
                    Color::Red
                } else if m.timeouts > 0 {
                    Color::Yellow
                } else {
                    Color::Magenta
                };
                (
                    format!(" DONE | S{:.0} ", m.score),
                    Style::default().bg(color).fg(Color::Black),
                )
            }
            None => (" ... ".to_string(), Style::default().fg(Color::DarkGray)),
        };
        pop_spans.push(Span::styled(text, style));
        pop_spans.push(Span::raw(" ")); // Spacing
    }

    let pop_paragraph = Paragraph::new(ratatui::text::Line::from(pop_spans))
        .block(
            Block::default()
                .title("Current Generation Population")
                .borders(Borders::ALL),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(pop_paragraph, stats_chunks[1]);

    // Weights Table
    if let Some(last_status) = app.history.last() {
        let rows = if let Some(w) = &last_status.current_weights_beam {
            format_beam_weights(w)
        } else if let Some(w) = &last_status.current_weights_rhea {
            format_rhea_weights(w)
        } else {
            vec![]
        };

        let table = Table::new(
            rows,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .header(Row::new(vec!["Parameter", "Value"]).style(Style::default().fg(Color::Yellow)))
        .block(
            Block::default()
                .title("Current Best Weights")
                .borders(Borders::ALL),
        )
        .column_spacing(1);
        f.render_widget(table, chunks[3]);
    }
}

fn format_beam_weights<'a>(w: &BeamScoringWeights) -> Vec<Row<'a>> {
    let debug_str = format!("{:?}", w);
    let content = debug_str
        .trim_start_matches("BeamScoringWeights {")
        .trim_end_matches("}");

    content
        .split(',')
        .map(|part| {
            let parts: Vec<&str> = part.split(':').collect();
            if parts.len() == 2 {
                Row::new(vec![
                    parts[0].trim().to_string(),
                    parts[1].trim().to_string(),
                ])
            } else {
                Row::new(vec![part.trim().to_string(), "".to_string()])
            }
        })
        .collect()
}

fn format_rhea_weights<'a>(w: &RheaScoringWeights) -> Vec<Row<'a>> {
    let debug_str = format!("{:?}", w);
    let content = debug_str
        .trim_start_matches("RheaScoringWeights {")
        .trim_end_matches("}");

    content
        .split(',')
        .map(|part| {
            let parts: Vec<&str> = part.split(':').collect();
            if parts.len() == 2 {
                Row::new(vec![
                    parts[0].trim().to_string(),
                    parts[1].trim().to_string(),
                ])
            } else {
                Row::new(vec![part.trim().to_string(), "".to_string()])
            }
        })
        .collect()
}
