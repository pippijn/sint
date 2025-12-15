use super::get_player_emoji;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget, Wrap},
};
use sint_core::types::{AttackEffect, GameState, HazardType, ItemType, MapLayout, SystemType};

const ROOM_BG_COLOR: Color = Color::Rgb(35, 35, 35);

#[derive(Clone, Copy, PartialEq)]
enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq)]
struct Door {
    side: Side,
    // Offset relative to the widget area.
    // If None, centered.
    // For Top/Bottom: X coordinate relative to left.
    // For Left/Right: Y coordinate relative to top.
    offset: Option<u16>,
}

pub struct MapWidget<'a> {
    pub state: Option<&'a GameState>,
    pub block: Option<Block<'a>>,
}

impl<'a> MapWidget<'a> {
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> Widget for MapWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let area = if let Some(block) = self.block {
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            let block = Block::default().title("Ship Map").borders(Borders::ALL);
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        };

        if let Some(state) = self.state {
            match state.layout {
                MapLayout::Star => render_star(state, area, buf),
                MapLayout::Torus => render_torus(state, area, buf),
            }
        } else {
            Paragraph::new("Waiting for state...")
                .style(Style::default().fg(Color::DarkGray))
                .render(area, buf);
        }
    }
}

struct RoomWidget<'a> {
    room_id: u32,
    state: &'a GameState,
    doors: Vec<Door>,
}

impl<'a> Widget for RoomWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let room = match self.state.map.rooms.get(&self.room_id) {
            Some(r) => r,
            None => return,
        };

        // Determine Border Style
        let is_targeted =
            self.state.enemy.next_attack.as_ref().is_some_and(|a| {
                a.target_room == Some(self.room_id) && a.effect != AttackEffect::Miss
            });

        let border_style = if is_targeted {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        // Render Block (Borders only, no bg)
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .border_type(BorderType::Rounded);

        let inner_area = block.inner(area);
        block.render(area, buf);

        // Fill background of interior
        for y in inner_area.y..inner_area.y + inner_area.height {
            for x in inner_area.x..inner_area.x + inner_area.width {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_bg(ROOM_BG_COLOR);
                }
            }
        }

        // Render Doors
        for door in &self.doors {
            match door.side {
                Side::Top => {
                    let cx = if let Some(off) = door.offset {
                        area.x + off
                    } else {
                        area.x + area.width / 2 - 1
                    };
                    let cy = area.y;

                    // 2-char wide opening at cx, cx+1
                    if cx > area.x && cx + 2 < area.x + area.width {
                        // Gap
                        for x in cx..=cx + 1 {
                            if let Some(cell) = buf.cell_mut((x, cy)) {
                                cell.set_symbol(" ").set_bg(ROOM_BG_COLOR);
                            }
                        }
                        // Corners
                        if cx > area.x {
                            buf.cell_mut((cx - 1, cy)).unwrap().set_symbol("╯");
                        }
                        if cx + 2 < area.x + area.width {
                            buf.cell_mut((cx + 2, cy)).unwrap().set_symbol("╰");
                        }
                    }
                }
                Side::Bottom => {
                    let cx = if let Some(off) = door.offset {
                        area.x + off
                    } else {
                        area.x + area.width / 2 - 1
                    };
                    let cy = area.y + area.height - 1;

                    if cx > area.x && cx + 2 < area.x + area.width {
                        // Gap
                        for x in cx..=cx + 1 {
                            if let Some(cell) = buf.cell_mut((x, cy)) {
                                cell.set_symbol(" ").set_bg(ROOM_BG_COLOR);
                            }
                        }
                        // Corners
                        if cx > area.x {
                            buf.cell_mut((cx - 1, cy)).unwrap().set_symbol("╮");
                        }
                        if cx + 2 < area.x + area.width {
                            buf.cell_mut((cx + 2, cy)).unwrap().set_symbol("╭");
                        }
                    }
                }
                Side::Left => {
                    let cx = area.x;
                    let cy = if let Some(off) = door.offset {
                        area.y + off
                    } else {
                        area.y + area.height / 2 - 1
                    };

                    if cy > area.y && cy + 2 < area.y + area.height {
                        // Gap
                        for y in cy..=cy + 1 {
                            if let Some(cell) = buf.cell_mut((cx, y)) {
                                cell.set_symbol(" ").set_bg(ROOM_BG_COLOR);
                            }
                        }
                        // Corners
                        if cy > area.y {
                            buf.cell_mut((cx, cy - 1)).unwrap().set_symbol("╯");
                        }
                        if cy + 2 < area.y + area.height {
                            buf.cell_mut((cx, cy + 2)).unwrap().set_symbol("╮");
                        }
                    }
                }
                Side::Right => {
                    let cx = area.x + area.width - 1;
                    let cy = if let Some(off) = door.offset {
                        area.y + off
                    } else {
                        area.y + area.height / 2 - 1
                    };

                    if cy > area.y && cy + 2 < area.y + area.height {
                        // Gap
                        for y in cy..=cy + 1 {
                            if let Some(cell) = buf.cell_mut((cx, y)) {
                                cell.set_symbol(" ").set_bg(ROOM_BG_COLOR);
                            }
                        }
                        // Corners
                        if cy > area.y {
                            buf.cell_mut((cx, cy - 1)).unwrap().set_symbol("╰");
                        }
                        if cy + 2 < area.y + area.height {
                            buf.cell_mut((cx, cy + 2)).unwrap().set_symbol("╭");
                        }
                    }
                }
            }
        }

        // Content
        let sys_icon = match room.system {
            Some(SystemType::Engine) => "🔧",
            Some(SystemType::Cannons) => "⚔️",
            Some(SystemType::Kitchen) => "🍳",
            Some(SystemType::Sickbay) => "🏥",
            Some(SystemType::Cargo) => "📦",
            Some(SystemType::Bridge) => "🎮",
            Some(SystemType::Bow) => "🏹",
            Some(SystemType::Dormitory) => "🛏️",
            Some(SystemType::Storage) => "🗄️",
            None => "  ",
        };

        // Calculate available width for name
        let inner_width = inner_area.width as usize;
        let max_name_len = inner_width.saturating_sub(7).max(1);

        let safe_name: String = room.name.as_str().chars().take(max_name_len).collect();

        let title_line = Line::from(vec![
            Span::styled(
                format!("{:02} ", self.room_id),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{:<width$} ", safe_name, width = max_name_len)),
            Span::raw(sys_icon),
        ]);

        let mut lines = vec![title_line];

        // System Health
        if room.system.is_some() {
            let hp_color = if room.system_health == 0 {
                Color::Red
            } else if room.system_health < sint_core::types::SYSTEM_HEALTH {
                Color::Yellow
            } else {
                Color::Green
            };
            lines.push(Line::from(vec![Span::styled(
                format!(
                    "HP: {}/{} ",
                    room.system_health,
                    sint_core::types::SYSTEM_HEALTH
                ),
                Style::default().fg(hp_color).add_modifier(Modifier::BOLD),
            )]));
        }

        // Players
        let players: Vec<_> = self
            .state
            .players
            .values()
            .filter(|p| p.room_id == self.room_id)
            .collect();

        let mut p_spans = Vec::new();
        for p in players {
            let emoji = get_player_emoji(&p.id);
            p_spans.push(Span::raw(emoji));
        }
        if !p_spans.is_empty() {
            lines.push(Line::from(p_spans));
        }

        // Hazards
        let mut h_spans = Vec::new();
        for h in &room.hazards {
            match h {
                HazardType::Fire => h_spans.push(Span::raw("🔥")),
                HazardType::Water => h_spans.push(Span::raw("💧")),
            }
        }

        use std::collections::BTreeMap;

        // Items
        let mut counts = BTreeMap::new();
        for item in &room.items {
            *counts.entry(item).or_insert(0) += 1;
        }

        let mut i_spans = Vec::new();
        for (item, count) in counts {
            let symbol = match item {
                ItemType::Peppernut => "🍪",
                ItemType::Extinguisher => "🧯",
                ItemType::Keychain => "🔑",
                ItemType::Wheelbarrow => "🛒",
                ItemType::Mitre => "🧢",
            };

            if count > 10 {
                i_spans.push(Span::raw(format!("{}x{} ", count, symbol)));
            } else {
                for _ in 0..count {
                    i_spans.push(Span::raw(symbol.to_string()));
                }
                i_spans.push(Span::raw(" "));
            }
        }

        // Layout inner lines
        if !h_spans.is_empty() {
            lines.push(Line::from(h_spans));
        }
        if !i_spans.is_empty() {
            lines.push(Line::from(i_spans));
        }

        let para = Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White).bg(ROOM_BG_COLOR));
        para.render(inner_area, buf);
    }
}

fn render_torus(state: &GameState, area: Rect, buf: &mut Buffer) {
    // 4x4 Grid Layout
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    let grid_rects: Vec<Vec<Rect>> = rows
        .iter()
        .map(|row| {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ])
                .split(*row)
                .to_vec()
        })
        .collect();

    // Map Room ID to Grid Coordinates (row, col) with specific border handling
    // Map Room ID to Grid Coordinates (row, col) with specific border handling
    // ...

    // Pass 2: Render with dynamic borders

    // Pass 2: Render with dynamic borders
    // Build map id -> (r,c)
    let mut pos_map = std::collections::HashMap::new();
    pos_map.insert(0, (0, 0));
    pos_map.insert(1, (0, 1));
    pos_map.insert(2, (0, 2));
    pos_map.insert(3, (0, 3));
    pos_map.insert(4, (1, 3));
    pos_map.insert(5, (2, 3));
    pos_map.insert(6, (3, 3));
    pos_map.insert(7, (3, 2));
    pos_map.insert(8, (3, 1));
    pos_map.insert(9, (3, 0));
    pos_map.insert(10, (2, 0));
    pos_map.insert(11, (1, 0));

    for (rid, (r, c)) in &pos_map {
        let rid = *rid;
        let room = match state.map.rooms.get(&rid) {
            Some(r) => r,
            None => continue,
        };

        let mut doors = Vec::new();

        for neighbor_id in &room.neighbors {
            if let Some((nr, nc)) = pos_map.get(neighbor_id) {
                if *nr == *r && *nc == *c + 1 {
                    doors.push(Door {
                        side: Side::Right,
                        offset: None,
                    });
                }
                if *nr == *r && *c > 0 && *nc == *c - 1 {
                    doors.push(Door {
                        side: Side::Left,
                        offset: None,
                    });
                }

                if *nc == *c && *nr == *r + 1 {
                    doors.push(Door {
                        side: Side::Bottom,
                        offset: None,
                    });
                }
                if *nc == *c && *r > 0 && *nr == *r - 1 {
                    doors.push(Door {
                        side: Side::Top,
                        offset: None,
                    });
                }
            }
        }

        let rect = grid_rects[*r][*c];
        RoomWidget {
            room_id: rid,
            state,
            doors,
        }
        .render(rect, buf);
    }
}

fn render_star(state: &GameState, area: Rect, buf: &mut Buffer) {
    // 3 Rows
    // Row 0: Top Rooms
    // Row 1: Hub (Center)
    // Row 2: Bottom Rooms

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    let hub_id = 0; // Assume 0 is hub
    let mut top_rooms = Vec::new();
    let mut bot_rooms = Vec::new();

    let mut other_ids: Vec<u32> = state.map.rooms.keys().filter(|&id| id != hub_id).collect();
    other_ids.sort();

    for (i, id) in other_ids.iter().enumerate() {
        if i % 2 == 0 {
            top_rooms.push(*id);
        } else {
            bot_rooms.push(*id);
        }
    }

    let col_count = top_rooms.len().max(bot_rooms.len());

    // Top Row
    let top_rects = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Ratio(1, col_count as u32); col_count])
        .split(rows[0]);

    let mut hub_doors = Vec::new();

    for (i, rid) in top_rooms.iter().enumerate() {
        // Connected to Hub (Bottom)
        let doors = vec![Door {
            side: Side::Bottom,
            offset: None,
        }];
        RoomWidget {
            room_id: *rid,
            state,
            doors,
        }
        .render(top_rects[i], buf);

        // Calculate offset for Hub relative to its own area
        // Hub is at rows[1].x
        // Room is at top_rects[i].x
        // Door center should be at Room center
        let room_center_x = top_rects[i].x + top_rects[i].width / 2 - 1;
        let hub_start_x = rows[1].x;
        // Ensure offset is positive (should be if layout is consistent)
        if room_center_x >= hub_start_x {
            let offset = room_center_x - hub_start_x;
            hub_doors.push(Door {
                side: Side::Top,
                offset: Some(offset),
            });
        }
    }

    // Bot Row
    let bot_rects = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Ratio(1, col_count as u32); col_count])
        .split(rows[2]);

    for (i, rid) in bot_rooms.iter().enumerate() {
        // Connected to Hub (Top)
        let doors = vec![Door {
            side: Side::Top,
            offset: None,
        }];
        RoomWidget {
            room_id: *rid,
            state,
            doors,
        }
        .render(bot_rects[i], buf);

        // Offset for Hub (Bottom)
        let room_center_x = bot_rects[i].x + bot_rects[i].width / 2 - 1;
        let hub_start_x = rows[1].x;
        if room_center_x >= hub_start_x {
            let offset = room_center_x - hub_start_x;
            hub_doors.push(Door {
                side: Side::Bottom,
                offset: Some(offset),
            });
        }
    }

    // Hub (Middle)
    RoomWidget {
        room_id: hub_id,
        state,
        doors: hub_doors,
    }
    .render(rows[1], buf);
}
