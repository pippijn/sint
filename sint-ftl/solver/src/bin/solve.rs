use clap::{Parser, ValueEnum};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
};
use sint_core::logic::GameLogic;
use sint_solver::replay;
use sint_solver::scoring::beam::BeamScoringWeights;
use sint_solver::scoring::rhea::RheaScoringWeights;
use sint_solver::search::beam::{BeamSearchConfig, beam_search};
use sint_solver::search::rhea::{RHEAConfig, rhea_search};
use sint_solver::search::{SearchNode, SearchProgress};
use sint_solver::tui::{
    log::LogWidget, map::MapWidget, players::PlayersWidget, score::ScoreWidget,
    situations::SituationsWidget, stats::StatsWidget,
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
    #[arg(short, long, default_value_t = 300)]
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
            rhea_search(&config, &weights, None::<fn(SearchProgress)>)
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
    failed: bool,
    start_time: Instant,
    final_duration: Option<Duration>,
}

impl SolverApp {
    fn new() -> Self {
        Self {
            progress: None,
            done: false,
            failed: false,
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
                rhea_search(&config, &weights, Some(callback))
            }
        };

        if let Some(s) = sol {
            save_solution(&s, &solver_args);
        }
    });

    let mut last_update = Instant::now();

    'mainloop: loop {
        terminal.draw(|f| ui(f, &app))?;

        let current_tick_rate = if app.done {
            Duration::from_secs(60)
        } else {
            Duration::from_millis(100)
        };

        // Rate Limit & Input Handling: Wait until tick_rate has passed
        let mut time_remaining = current_tick_rate
            .checked_sub(last_update.elapsed())
            .unwrap_or(Duration::from_secs(0));

        while time_remaining > Duration::from_secs(0) {
            if crossterm::event::poll(time_remaining)?
                && let Event::Key(key) = crossterm::event::read()?
                && let KeyCode::Char('q') = key.code
            {
                break 'mainloop;
            }
            time_remaining = current_tick_rate
                .checked_sub(last_update.elapsed())
                .unwrap_or(Duration::from_secs(0));
        }

        last_update = Instant::now();

        while let Ok(p) = rx.try_recv() {
            if p.is_done && !app.done {
                app.done = true;
                app.failed = p.failed;
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

    // Data prep
    let current_node = app.progress.as_ref().map(|p| &p.node);
    let current_state = current_node.map(|n| &n.state);

    // Stats Widget
    let (step, hull, boss, score) = if let Some(p) = &app.progress {
        (
            p.step,
            p.node.state.hull_integrity,
            p.node.state.enemy.hp,
            p.node.score.total,
        )
    } else {
        (0, 0, 0, 0.0)
    };

    let (shields, evasion) = if let Some(s) = current_state {
        (s.shields_active, s.evasion_active)
    } else {
        (false, false)
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
        failed: app.failed,
        shields_active: shields,
        evasion_active: evasion,
    };
    f.render_widget(stats, chunks[0]);

    // Content: Map + Sidebar (Players + Situations + Log)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
        .split(chunks[1]);

    let side_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(10), // Players
                Constraint::Length(12), // Situations + Score
                Constraint::Min(0),     // Log
            ]
            .as_ref(),
        )
        .split(main_chunks[1]);

    let mid_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(side_chunks[1]);

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

    // Situations Widget (Middle Right Left)
    let situations = SituationsWidget {
        state: current_state,
    };
    f.render_widget(situations, mid_chunks[0]);

    // Score Widget (Middle Right Right)
    let score_details = current_node.map(|n| n.score);
    let score_widget = ScoreWidget {
        score: score_details,
    };
    f.render_widget(score_widget, mid_chunks[1]);

    // Log Widget (Bottom Right)
    let log = LogWidget { current_node };
    f.render_widget(log, side_chunks[2]);
}
