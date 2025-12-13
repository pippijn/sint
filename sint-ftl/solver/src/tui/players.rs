use super::{get_player_color, get_player_emoji};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Widget},
};
use sint_core::types::{GameState, ItemType, PlayerStatus};

pub struct PlayersWidget<'a> {
    pub state: Option<&'a GameState>,
}

impl<'a> Widget for PlayersWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Crew Status")
            .border_style(Style::default().fg(Color::Yellow));

        let mut items = Vec::new();

        if let Some(state) = self.state {
            for player in state.players.values() {
                // Color code the player ID/Name
                let name_color = get_player_color(&player.id);
                let emoji = get_player_emoji(&player.id);

                let mut spans = vec![
                    Span::raw(format!("{} ", emoji)),
                    Span::styled(
                        format!("{:<3}", player.id),
                        Style::default().fg(name_color).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!("(R{:02}) | ", player.room_id)),
                ];

                // HP
                spans.push(Span::styled("HP:", Style::default().fg(Color::DarkGray)));
                for i in 0..3 {
                    if i < player.hp {
                        spans.push(Span::styled("♥", Style::default().fg(Color::Red)));
                    } else {
                        spans.push(Span::styled("♡", Style::default().fg(Color::DarkGray)));
                    }
                }
                spans.push(Span::raw(" "));

                // AP
                spans.push(Span::styled("AP:", Style::default().fg(Color::DarkGray)));
                for i in 0..2 {
                    if i < player.ap {
                        spans.push(Span::styled("★", Style::default().fg(Color::Yellow)));
                    } else {
                        spans.push(Span::styled("☆", Style::default().fg(Color::DarkGray)));
                    }
                }
                spans.push(Span::raw(" | "));

                // Status
                if !player.status.is_empty() {
                    for status in &player.status {
                        let (icon, color) = match status {
                            PlayerStatus::Fainted => ("💀", Color::Red),
                            PlayerStatus::Silenced => ("😶", Color::Gray),
                        };
                        spans.push(Span::styled(
                            format!("{} ", icon),
                            Style::default().fg(color),
                        ));
                    }
                    spans.push(Span::raw("| "));
                }

                // Inventory
                if !player.inventory.is_empty() {
                    for item in &player.inventory {
                        let symbol = match item {
                            ItemType::Peppernut => "🍪",
                            ItemType::Extinguisher => "🧯",
                            ItemType::Keychain => "🔑",
                            ItemType::Wheelbarrow => "🛒",
                            ItemType::Mitre => "🧢",
                        };
                        spans.push(Span::raw(format!("{} ", symbol)));
                    }
                } else {
                    spans.push(Span::styled(
                        "(empty)",
                        Style::default().fg(Color::DarkGray),
                    ));
                }

                items.push(ListItem::new(Line::from(spans)));
            }
        } else {
            items.push(ListItem::new("Waiting for state..."));
        }

        List::new(items).block(block).render(area, buf);
    }
}
