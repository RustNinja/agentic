use session_protocol::WireStatusKind;

pub struct SessionEngine {
    label: String,
}

impl SessionEngine {
    pub fn connect(label: &str) -> Self {
        Self {
            label: normalize_label(label),
        }
    }

    pub fn status(&self) -> SessionStatus {
        SessionStatus::new(&self.label, WireStatusKind::Connected)
    }

    pub fn dead_debug(&self) -> String {
        format!("dead-core:{}", self.label)
    }
}

pub struct SessionStatus {
    label: String,
    kind: WireStatusKind,
}

impl SessionStatus {
    pub fn new(label: &str, kind: WireStatusKind) -> Self {
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

    pub fn dead_label(&self) -> String {
        format!("dead-status:{}", self.label)
    }
}

fn normalize_label(label: &str) -> String {
    label.trim().to_ascii_lowercase()
}

pub fn dead_live_core(label: &str) -> String {
    SessionEngine::connect(label).dead_debug()
}

