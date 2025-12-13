use crate::scoring::ScoreDetails;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct ScoreWidget {
    pub score: Option<ScoreDetails>,
}

impl Widget for ScoreWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Score Breakdown");

        let text = if let Some(s) = self.score {
            let values = [
                ("Total:", s.total),
                ("Vitals:", s.vitals),
                ("Hazards:", s.hazards),
                ("Offense:", s.offense),
                ("Panic:", s.panic),
                ("Logistics:", s.logistics),
                ("Situation:", s.situations),
                ("Threats:", s.threats),
                ("Progress:", s.progression),
                ("AntiOsc:", s.anti_oscillation),
            ];

            let formatted_values: Vec<(&str, String)> = values
                .iter()
                .map(|(label, val)| (*label, format!("{:.1}", val)))
                .collect();

            let max_val_len = formatted_values
                .iter()
                .map(|(_, val)| val.len())
                .max()
                .unwrap_or(0);

            let mut output = String::new();
            for (i, (label, val)) in formatted_values.iter().enumerate() {
                if i > 0 {
                    output.push('\n');
                }
                output.push_str(&format!(
                    "{:<10} {:>width$}",
                    label,
                    val,
                    width = max_val_len
                ));
            }
            output
        } else {
            "No score available".to_string()
        };

        Paragraph::new(text)
            .block(block)
            .style(Style::default().fg(Color::Yellow))
            .render(area, buf);
    }
}
