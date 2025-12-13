use crate::search::SearchNode;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Widget},
};
use std::sync::Arc;

pub struct LogWidget<'a> {
    pub current_node: Option<&'a Arc<SearchNode>>,
}

impl<'a> Widget for LogWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut items = Vec::new();
        let mut title = "Best Trajectory".to_string();

        if let Some(node) = self.current_node {
            title = format!("Best Trajectory ({} actions)", node.history_len);
            let history = node.get_recent_history(20);

            let start_idx = node.history_len.saturating_sub(20);

            for (i, (pid, act)) in history.iter().enumerate() {
                items.push(ListItem::new(format!(
                    "{}: {} -> {:?}",
                    start_idx + i + 1,
                    pid,
                    act
                )));
            }
        } else {
            items.push(ListItem::new("Waiting for data..."));
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::White));

        List::new(items).block(block).render(area, buf);
    }
}
