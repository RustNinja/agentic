#[derive(Clone, Copy)]
pub enum WireStatusKind {
    Connected,
    Disconnected,
    Reconnecting,
}

pub struct WireSession {
    label: String,
    kind: WireStatusKind,
}

impl WireSession {
    pub fn from_status(label: &str, kind: WireStatusKind) -> Self {
        Self {
            label: label.to_string(),
            kind,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn kind(&self) -> WireStatusKind {
        self.kind
    }

    pub fn dead_summary(&self) -> String {
        format!("dead-wire:{}", self.label)
    }
}

pub fn dead_live_protocol(label: &str) -> WireSession {
    WireSession::from_status(label, WireStatusKind::Disconnected)
}

