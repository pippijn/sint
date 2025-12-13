use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::time::Duration;

pub struct StatsWidget {
    pub step: usize,
    pub hull: i32,
    pub boss_hp: i32,
    pub score: f64,
    pub duration: Duration,
    pub is_done: bool,
    pub failed: bool,
    pub shields_active: bool,
    pub evasion_active: bool,
}

impl Widget for StatsWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let status_txt = if self.failed {
            "FAILED - Press 'q' to quit"
        } else if self.is_done {
            "DONE - Press 'q' to quit"
        } else {
            "SEARCHING..."
        };

        let defenses = if self.shields_active && self.evasion_active {
            " [SHIELDS+EVASION]"
        } else if self.shields_active {
            " [SHIELDS]"
        } else if self.evasion_active {
            " [EVASION]"
        } else {
            ""
        };

        let stats_text = format!(
            "Solver TUI | Step: {} | Hull: {} | Boss: {} | Score: {:.0} | Time: {:.1}s | {}{}",
            self.step,
            self.hull,
            self.boss_hp,
            self.score,
            self.duration.as_secs_f64(),
            status_txt,
            defenses
        );

        let color = if self.failed {
            Color::Red
        } else if self.is_done {
            Color::Green
        } else {
            Color::Cyan
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Status")
            .border_style(Style::default().fg(color));

        Paragraph::new(stats_text)
            .style(Style::default().fg(color))
            .block(block)
            .render(area, buf);
    }
}
