use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Widget},
};
use sint_core::types::GameState;

pub struct SituationsWidget<'a> {
    pub state: Option<&'a GameState>,
}

impl<'a> Widget for SituationsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Active Situations")
            .border_style(Style::default().fg(Color::Magenta));

        let mut items = Vec::new();

        if let Some(state) = self.state {
            if state.active_situations.is_empty() {
                items.push(ListItem::new(Span::styled(
                    "None",
                    Style::default().fg(Color::DarkGray),
                )));
            } else {
                for card in &state.active_situations {
                    items.push(ListItem::new(Span::raw(format!("- {}", card.title))));
                }
            }
        } else {
            items.push(ListItem::new("Waiting for state..."));
        }

        List::new(items).block(block).render(area, buf);
    }
}
