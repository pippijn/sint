use ratatui::style::Color;

pub mod log;
pub mod map;
pub mod players;
pub mod situations;
pub mod stats;

pub fn get_player_emoji(id: &str) -> &'static str {
    match id {
        "P1" => "👺",
        "P2" => "🤖",
        "P3" => "🐸",
        "P4" => "😺",
        "P5" => "😈",
        "P6" => "👻",
        _ => "👤",
    }
}

pub fn get_player_color(id: &str) -> Color {
    match id {
        "P1" => Color::Red,
        "P2" => Color::Blue,
        "P3" => Color::Green,
        "P4" => Color::Yellow,
        "P5" => Color::Magenta,
        "P6" => Color::Cyan,
        _ => Color::White,
    }
}
