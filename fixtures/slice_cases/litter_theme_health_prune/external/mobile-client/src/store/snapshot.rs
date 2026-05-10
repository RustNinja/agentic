#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerHealthSnapshot {
    Disconnected,
    Connecting,
    Connected,
    Unresponsive,
    Unknown(String),
}

impl ServerHealthSnapshot {
    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }

    pub fn dead_label(&self) -> &'static str {
        "dead-snapshot"
    }
}

pub struct AppSnapshot {
    pub health: ServerHealthSnapshot,
}

pub enum DeadHealthSnapshot {
    Offline,
}

pub fn dead_snapshot() -> DeadHealthSnapshot {
    DeadHealthSnapshot::Offline
}

