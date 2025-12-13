use clap::{Parser, ValueEnum};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Frame, Terminal,
};
use sint_core::logic::GameLogic;
use sint_solver::replay;
use sint_solver::scoring::beam::BeamScoringWeights;
use sint_solver::scoring::rhea::RheaScoringWeights;
use sint_solver::search::beam::{beam_search, BeamSearchConfig, SearchProgress};
use sint_solver::search::rhea::{rhea_search, RHEAConfig};
use sint_solver::search::SearchNode;
use sint_solver::tui::{
    log::LogWidget, map::MapWidget, players::PlayersWidget, stats::StatsWidget,
};
use std::fs::File;
use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(ValueEnum, Clone, Debug, Default)]
enum Strategy {
    #[default]
    Beam,
    Rhea,
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Search Strategy
    #[arg(long, value_enum, default_value_t = Strategy::Beam)]
    strategy: Strategy,

    /// Beam Width (Number of states to keep per step)
    #[arg(short, long, default_value_t = 200)]
    beam_width: usize,

    /// RHEA Horizon
    #[arg(long, default_value_t = 10)]
    rhea_horizon: usize,

    /// RHEA Generations per step
    #[arg(long, default_value_t = 50)]
    rhea_generations: usize,

    /// RHEA Population
    #[arg(long, default_value_t = 20)]
    rhea_pop: usize,

    /// Number of players
    #[arg(short, long, default_value_t = 6)]
    players: usize,

    /// Random Seed
    #[arg(long, default_value_t = 12345)]
    seed: u64,

    /// Max steps (actions/depth)
    #[arg(short, long, default_value_t = 3000)]
    steps: usize,

    /// Time limit in seconds
    #[arg(short, long, default_value_t = 300)]
    time_limit: u64,

    /// Output file for trajectory
    #[arg(short, long, default_value = "/tmp/solve_output.txt")]
    output: Option<String>,

    /// Enable TUI mode
    #[arg(long)]
    tui: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.tui {
        run_tui(args)?;
    } else {
        run_cli(args);
    }
    Ok(())
}

fn save_solution(sol: &SearchNode, args: &Args) {
    let player_ids: Vec<String> = (0..args.players).map(|i| format!("P{}", i + 1)).collect();

    // Logic extracted from run_cli
    if args.tui {
        // In TUI we don't print to stdout, only log to file if requested
    } else {
        println!("\n=== BEST RESULT ===");
        println!("Final Phase: {:?}", sol.state.phase);
        println!("Hull: {}", sol.state.hull_integrity);
        println!("Boss HP: {}", sol.state.enemy.hp);
    }

    let initial_print = GameLogic::new_game(player_ids, args.seed);
    let history = sol
        .get_history()
        .into_iter()
        .map(|(p, a)| (p.clone(), a.clone()))
        .collect();
    let output_lines = replay::format_trajectory(initial_print, history);

    if let Some(path) = &args.output {
        let mut file = File::create(path).expect("Unable to create output file");

        // Write logic
        for chunk in output_lines {
            write!(file, "{}", chunk).expect("Unable to write data");
        }

        if !args.tui {
            println!("\n✅ Trajectory written to '{}'", path);
        }
    } else if !args.tui {
        println!("\n📜 Trajectory:");
        for line in output_lines {
            print!("{}", line);
        }
    }
}

fn run_cli(args: Args) {
    let sol = match args.strategy {
        Strategy::Beam => {
            let weights = BeamScoringWeights::default();
            let config = BeamSearchConfig {
                players: args.players,
                seed: args.seed,
                width: args.beam_width,
                steps: args.steps,
                time_limit: args.time_limit,
                verbose: true,
            };
            beam_search(&config, &weights, None::<fn(SearchProgress)>)
        }
        Strategy::Rhea => {
            let weights = RheaScoringWeights::default();
            let config = RHEAConfig {
                players: args.players,
                seed: args.seed,
                horizon: args.rhea_horizon,
                generations: args.rhea_generations,
                population_size: args.rhea_pop,
                max_steps: args.steps,
                time_limit: args.time_limit,
                verbose: true,
            };
            rhea_search(&config, &weights)
        }
    };

    if let Some(sol) = sol {
        save_solution(&sol, &args);
    } else {
        println!("❌ No solution found.");
    }
}

struct SolverApp {
    progress: Option<SearchProgress>,
    done: bool,
    start_time: Instant,
    final_duration: Option<Duration>,
}

impl SolverApp {
    fn new() -> Self {
        Self {
            progress: None,
            done: false,
            start_time: Instant::now(),
            final_duration: None,
        }
    }
}

fn run_tui(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    // Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = SolverApp::new();
    let (tx, rx) = mpsc::channel();

    // Spawn Solver Thread
    let solver_args = args.clone(); // Clone args for thread
    thread::spawn(move || {
        // Progress Callback
        let callback = move |p: SearchProgress| {
            let _ = tx.send(p);
        };

        let sol = match solver_args.strategy {
            Strategy::Beam => {
                let weights = BeamScoringWeights::default();
                let config = BeamSearchConfig {
                    players: solver_args.players,
                    seed: solver_args.seed,
                    width: solver_args.beam_width,
                    steps: solver_args.steps,
                    time_limit: solver_args.time_limit,
                    verbose: false, // Silence stdout in TUI
                };
                beam_search(&config, &weights, Some(callback))
            }
            Strategy::Rhea => {
                let weights = RheaScoringWeights::default();
                let config = RHEAConfig {
                    players: solver_args.players,
                    seed: solver_args.seed,
                    horizon: solver_args.rhea_horizon,
                    generations: solver_args.rhea_generations,
                    population_size: solver_args.rhea_pop,
                    max_steps: solver_args.steps,
                    time_limit: solver_args.time_limit,
                    verbose: false,
                };
                rhea_search(&config, &weights)
            }
        };

        if let Some(s) = sol {
            save_solution(&s, &solver_args);
        }
    });

    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = if app.done {
            Duration::from_secs(60) // Wait for user input (e.g. 'q'), stop spinning
        } else {
            tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0))
        };

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = crossterm::event::read()? {
                if let KeyCode::Char('q') = key.code {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        while let Ok(p) = rx.try_recv() {
            if p.is_done && !app.done {
                app.done = true;
                app.final_duration = Some(app.start_time.elapsed());
            }
            app.progress = Some(p);
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    Ok(())
}

fn ui(f: &mut Frame, app: &SolverApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Stats
                Constraint::Min(0),    // Content
            ]
            .as_ref(),
        )
        .split(f.area());

    // Stats Widget
    let (step, hull, boss, score) = if let Some(p) = &app.progress {
        (p.step, p.hull, p.boss_hp, p.best_score)
    } else {
        (0, 0, 0, 0.0)
    };

    let duration = app
        .final_duration
        .unwrap_or_else(|| app.start_time.elapsed());

    let stats = StatsWidget {
        step,
        hull,
        boss_hp: boss,
        score,
        duration,
        is_done: app.done,
    };
    f.render_widget(stats, chunks[0]);

    // Content: Map + Sidebar (Players + Log)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
        .split(chunks[1]);

    let side_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Min(0)].as_ref())
        .split(main_chunks[1]);

    // Data prep
    let current_node = app
        .progress
        .as_ref()
        .and_then(|p| p.current_best_node.as_ref());
    let current_state = current_node.map(|n| &n.state);

    // Map Widget (Left)
    let map = MapWidget {
        state: current_state,
    };
    f.render_widget(map, main_chunks[0]);

    // Players Widget (Top Right)
    let players = PlayersWidget {
        state: current_state,
    };
    f.render_widget(players, side_chunks[0]);

    // Log Widget (Bottom Right)
    let log = LogWidget { current_node };
    f.render_widget(log, side_chunks[1]);
}
