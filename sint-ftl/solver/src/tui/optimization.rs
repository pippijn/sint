use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Paragraph, Row, Table, Widget},
};

use crate::scoring::ScoreDetails;
use sint_core::types::GameState;

/// A widget that renders a horizontal ribbon of color representing a game's state over time.
/// Each cell in the ribbon represents one round.
pub struct ScoreRibbon<'a> {
    /// Normalized scores/health per round (0.0 to 1.0)
    pub data: &'a [f32],
    /// Optional label for the ribbon (e.g., individual index)
    pub label: &'a str,
    /// Maximum number of rounds to display
    pub max_rounds: usize,
}

impl<'a> Widget for ScoreRibbon<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 1 || area.width < 1 {
            return;
        }

        // Render label if there is space
        let data_start_x = if !self.label.is_empty() && area.width > 10 {
            buf.set_stringn(
                area.x,
                area.y,
                self.label,
                8,
                Style::default().fg(Color::Gray),
            );
            area.x + 10
        } else {
            area.x
        };

        let available_width = area.width.saturating_sub(data_start_x - area.x);
        let rounds_to_show = self
            .data
            .len()
            .min(self.max_rounds)
            .min(available_width as usize);

        for i in 0..rounds_to_show {
            let val = self.data[i];

            // Map 0.0-1.0 to RGB: Red (0.0) -> Yellow (0.5) -> Green (1.0)
            let color = if val < 0.5 {
                // Red to Yellow
                let r = 255;
                let g = (val * 2.0 * 255.0) as u8;
                Color::Rgb(r, g, 0)
            } else {
                // Yellow to Green
                let r = ((1.0 - (val - 0.5) * 2.0) * 255.0) as u8;
                let g = 255;
                Color::Rgb(r, g, 0)
            };

            buf[(data_start_x + i as u16, area.y)]
                .set_char('█')
                .set_style(Style::default().fg(color));
        }
    }
}

/// A dense grid representing parameter values across the population
pub struct GenomeMosaic<'a> {
    /// Individuals (Rows) x Parameters (Cols)
    pub population: &'a [Vec<f64>],
    /// Parameter names for the header
    pub param_names: &'a [String],
    pub block: Option<Block<'a>>,
}

impl<'a> GenomeMosaic<'a> {
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> Widget for GenomeMosaic<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = if let Some(block) = self.block {
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };

        if area.height < 2 || area.width < 5 || self.population.is_empty() {
            return;
        }

        let label_width = 4; // "C## "
        let header_height = 1;
        let col_width = 1; // Very dense

        // 1. Header (Parameter Indices)
        for col_idx in 0..self.param_names.len() {
            let x = area.x + label_width + (col_idx as u16 * col_width);
            if x >= area.x + area.width {
                break;
            }

            // Show every 5th or 10th index to avoid clutter
            if col_idx % 5 == 0 {
                let idx_str = format!("{}", col_idx % 100);
                buf.set_string(x, area.y, idx_str, Style::default().fg(Color::Gray));
            }
        }

        // 2. Rows
        for (row_idx, genome) in self
            .population
            .iter()
            .enumerate()
            .take(area.height as usize - header_height)
        {
            let y = area.y + header_height as u16 + row_idx as u16;

            // Chromosome Label
            let label = format!("C{:02}", row_idx);
            buf.set_string(area.x, y, label, Style::default().fg(Color::Gray));

            for (col_idx, &val) in genome.iter().enumerate() {
                let x = area.x + label_width + (col_idx as u16 * col_width);
                if x >= area.x + area.width {
                    break;
                }

                // Normalize value for color mapping. Multipliers are usually around 1.0.
                // We'll map 0.0 -> Blue, 1.0 -> White, 2.0+ -> Red
                let color = if val < 1.0 {
                    let b = 255;
                    let r = (val * 255.0) as u8;
                    let g = r;
                    Color::Rgb(r, g, b)
                } else {
                    let r = 255;
                    let b = ((2.0 - val).max(0.0) * 255.0) as u8;
                    let g = b;
                    Color::Rgb(r, g, b)
                };

                buf[(x, y)]
                    .set_char('█')
                    .set_style(Style::default().fg(color));
            }
        }
    }
}

/// A compact grid showing the status of every game (Chromosome x Seed)
pub struct SeedGauntlet<'a> {
    /// Individual Index -> Seed Index -> Status
    /// 0: Pending, 1: Running, 2: Win, 3: Loss, 4: Panic, 5: Timeout
    pub status: &'a [Vec<u8>],
    pub seed_labels: &'a [u64],
    pub block: Option<Block<'a>>,
}

impl<'a> SeedGauntlet<'a> {
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> Widget for SeedGauntlet<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = if let Some(block) = self.block {
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };

        if self.status.is_empty() {
            return;
        }

        // 1. Render Seed Headers (Top)
        // We'll show the indices 01, 02... and maybe the last 3 digits of the seed if there's space
        let col_width = 3; // Space for " ##"
        let header_height = 1;
        let label_width = 4; // Space for "C## "

        if area.height < header_height + 1 || area.width < label_width + 1 {
            return;
        }

        for c_idx in 0..self.seed_labels.len() {
            let x = area.x + label_width + (c_idx as u16 * col_width);
            if x + col_width > area.x + area.width {
                break;
            }

            let seed_str = format!("{:02}", c_idx);
            buf.set_string(x + 1, area.y, seed_str, Style::default().fg(Color::Gray));
        }

        // 2. Render Rows (Chromosome Label + Status Cells)
        for (r_idx, row) in self
            .status
            .iter()
            .enumerate()
            .take(area.height as usize - header_height as usize)
        {
            let y = area.y + header_height + r_idx as u16;

            // Chromosome Label
            let label = format!("C{:02}", r_idx);
            buf.set_string(area.x, y, label, Style::default().fg(Color::Gray));

            for (c_idx, &s) in row.iter().enumerate() {
                let x = area.x + label_width + (c_idx as u16 * col_width);
                if x + col_width > area.x + area.width {
                    break;
                }

                let (ch, color) = match s {
                    1 => ('⠶', Color::Blue),     // Running
                    2 => ('█', Color::Green),    // Win
                    3 => ('█', Color::Red),      // Loss
                    4 => ('!', Color::Magenta),  // Panic
                    5 => ('█', Color::Yellow),   // Timeout
                    _ => ('.', Color::DarkGray), // Pending
                };

                buf[(x + 1, y)]
                    .set_char(ch)
                    .set_style(Style::default().fg(color));
            }
        }
    }
}

/// Header showing optimization progress and system stats
pub struct OptimizationHeader<'a> {
    pub strategy: &'a str,
    pub target: &'a str,
    pub generation: usize,
    pub max_generations: usize,
    pub population: usize,
    pub status: &'a str,
    pub games_done: usize,
    pub total_games: usize,
    pub games_pending: usize,
    pub games_running: usize,
    pub inds_done: usize,
    pub cpu_usage: f32,
    pub mem_proc: f64,
    pub mem_used: f64,
    pub mem_total: f64,
}

impl<'a> Widget for OptimizationHeader<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let header_chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                Constraint::Min(0),     // Left: Progress
                Constraint::Length(30), // Right: System Stats
            ])
            .split(area);

        // Left Header: Workload
        let games_pct = if self.total_games > 0 {
            (self.games_done as f64 / self.total_games as f64) * 100.0
        } else {
            0.0
        };

        let left_text = format!(
            "SINT OPTIMIZER | Strat: {} | Target: {} | Gen: {}/{} | Pop: {} | {}\n\
             WORKLOAD: Games: {}/{} ({:.1}%) ({} Pending, {} Running) | Genomes: {}/{} Completed",
            self.strategy,
            self.target,
            self.generation,
            self.max_generations,
            self.population,
            self.status,
            self.games_done,
            self.total_games,
            games_pct,
            self.games_pending,
            self.games_running,
            self.inds_done,
            self.population,
        );

        Paragraph::new(left_text)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(
                ratatui::widgets::Borders::LEFT
                    | ratatui::widgets::Borders::TOP
                    | ratatui::widgets::Borders::BOTTOM,
            ))
            .render(header_chunks[0], buf);

        // Right Header: System Stats
        let right_text = format!(
            "CPU: {:>5.1}%\nP:{:>4.1}G | S:{:>4.1}/{:>2.0}G",
            self.cpu_usage, self.mem_proc, self.mem_used, self.mem_total
        );

        Paragraph::new(right_text)
            .alignment(Alignment::Right)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(
                ratatui::widgets::Borders::RIGHT
                    | ratatui::widgets::Borders::TOP
                    | ratatui::widgets::Borders::BOTTOM,
            ))
            .render(header_chunks[1], buf);
    }
}

/// Detailed live statistics for a single game simulation
pub struct LiveStats<'a> {
    pub state: &'a GameState,
    pub score: f64,
    pub label: &'a str,
}

impl<'a> Widget for LiveStats<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let stats_text = format!(
            "Round: {}\nHull: {}/20\nBoss HP: {}/{}\nPhase: {:?}\nScore: {:.1}",
            self.state.turn_count,
            self.state.hull_integrity,
            self.state.enemy.hp,
            self.state.enemy.max_hp,
            self.state.phase,
            self.score
        );

        Paragraph::new(stats_text)
            .block(
                Block::default()
                    .title(format!("Live Stats {}", self.label))
                    .borders(ratatui::widgets::Borders::ALL),
            )
            .render(area, buf);
    }
}

/// Table breaking down the score components
pub struct ScoreBreakdown<'a> {
    pub score: &'a ScoreDetails,
    pub label: &'a str,
}

impl<'a> Widget for ScoreBreakdown<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let s = self.score;
        let v_str = format!("{:.1}", s.vitals);
        let h_str = format!("{:.1}", s.hazards);
        let o_str = format!("{:.1}", s.offense);
        let p_str = format!("{:.1}", s.panic);
        let l_str = format!("{:.1}", s.logistics);
        let s_str = format!("{:.1}", s.situations);
        let t_str = format!("{:.1}", s.threats);
        let pr_str = format!("{:.1}", s.progression);
        let ao_str = format!("{:.1}", s.anti_oscillation);
        let tot_str = format!("{:.1}", s.total);

        let rows = vec![
            Row::new(vec!["Vitals", v_str.as_str()]),
            Row::new(vec!["Hazards", h_str.as_str()]),
            Row::new(vec!["Offense", o_str.as_str()]),
            Row::new(vec!["Panic", p_str.as_str()]),
            Row::new(vec!["Logistics", l_str.as_str()]),
            Row::new(vec!["Situations", s_str.as_str()]),
            Row::new(vec!["Threats", t_str.as_str()]),
            Row::new(vec!["Progression", pr_str.as_str()]),
            Row::new(vec!["Anti-Osc", ao_str.as_str()]),
            Row::new(vec!["TOTAL", tot_str.as_str()]).style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Yellow),
            ),
        ];

        Table::new(rows, [Constraint::Length(15), Constraint::Min(0)])
            .block(
                Block::default()
                    .title(format!("Score Breakdown {}", self.label))
                    .borders(ratatui::widgets::Borders::ALL),
            )
            .render(area, buf);
    }
}

/// A ticker showing the last few actions taken in a simulation
pub struct ActionTicker<'a> {
    pub history: &'a [String],
    pub label: &'a str,
}

impl<'a> Widget for ActionTicker<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let ticker_text = self.history.join("\n");
        Paragraph::new(ticker_text)
            .block(
                Block::default()
                    .title(format!("Action Ticker {}", self.label))
                    .borders(ratatui::widgets::Borders::ALL),
            )
            .render(area, buf);
    }
}

/// Summary of the best and average performance in a generation
pub struct GenerationSummary<'a> {
    pub wins: usize,
    pub losses: usize,
    pub timeouts: usize,
    pub best_score: f64,
    pub avg_score: f64,
    pub is_loading: bool,
    pub block: Option<Block<'a>>,
}

impl<'a> GenerationSummary<'a> {
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> Widget for GenerationSummary<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = if let Some(block) = self.block {
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };

        if self.is_loading {
            Paragraph::new("\n\n  Waiting for first\n  generation to\n  complete...")
                .render(area, buf);
            return;
        }

        let wins = self.wins.to_string();
        let losses = self.losses.to_string();
        let timeouts = self.timeouts.to_string();
        let best_score = format!("{:.1}", self.best_score);
        let avg_score = format!("{:.1}", self.avg_score);

        let rows = vec![
            Row::new(vec!["Wins", wins.as_str()]).style(Style::default().fg(Color::Green)),
            Row::new(vec!["Losses", losses.as_str()]).style(Style::default().fg(Color::Red)),
            Row::new(vec!["Timeouts", timeouts.as_str()]).style(Style::default().fg(Color::Yellow)),
            Row::new(vec!["Best Score", best_score.as_str()]),
            Row::new(vec!["Avg Score", avg_score.as_str()]),
        ];

        Table::new(rows, [Constraint::Length(15), Constraint::Min(0)]).render(area, buf);
    }
}
