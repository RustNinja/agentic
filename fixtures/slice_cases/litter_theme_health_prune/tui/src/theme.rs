use mobile_client::store::ServerHealthSnapshot;
use opensourced::opensourced;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red,
    Yellow,
    Green,
    DarkGray,
    Cyan,
}

impl Color {
    pub fn name(self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Yellow => "yellow",
            Self::Green => "green",
            Self::DarkGray => "dark-gray",
            Self::Cyan => "cyan",
        }
    }

    pub fn dead_name(self) -> &'static str {
        "dead-color"
    }
}

pub const FG_DIM: Color = Color::DarkGray;
pub const ERROR: Color = Color::Red;
pub const WARNING: Color = Color::Yellow;
pub const SUCCESS: Color = Color::Green;
pub const ACCENT: Color = Color::Cyan;

pub fn accent() -> Color {
    ACCENT
}

#[opensourced]
pub fn health_color(health: &ServerHealthSnapshot) -> Color {
    match health {
        ServerHealthSnapshot::Connected => SUCCESS,
        ServerHealthSnapshot::Connecting => WARNING,
        ServerHealthSnapshot::Disconnected => ERROR,
        ServerHealthSnapshot::Unresponsive => ERROR,
        ServerHealthSnapshot::Unknown(_) => FG_DIM,
    }
}

#[opensourced]
pub fn health_symbol(health: &ServerHealthSnapshot) -> &'static str {
    match health {
        ServerHealthSnapshot::Connected => "connected",
        ServerHealthSnapshot::Connecting => "connecting",
        ServerHealthSnapshot::Disconnected => "disconnected",
        ServerHealthSnapshot::Unresponsive => "unresponsive",
        ServerHealthSnapshot::Unknown(_) => "unknown",
    }
}

pub fn dead_status_label(health: &ServerHealthSnapshot) -> &'static str {
    match health {
        ServerHealthSnapshot::Connected => ACCENT.dead_name(),
        _ => "dead",
    }
}

