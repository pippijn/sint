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
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Best Trajectory")
            .border_style(Style::default().fg(Color::White));

        let mut items = Vec::new();
        if let Some(node) = self.current_node {
            let history = node.get_history();
            // Show last 20 actions
            let start = if history.len() > 20 {
                history.len() - 20
            } else {
                0
            };
            for (i, (pid, act)) in history.iter().enumerate().skip(start) {
                items.push(ListItem::new(format!("{}: {} -> {:?}", i + 1, pid, act)));
            }
        } else {
            items.push(ListItem::new("Waiting for data..."));
        }

        List::new(items).block(block).render(area, buf);
    }
}
