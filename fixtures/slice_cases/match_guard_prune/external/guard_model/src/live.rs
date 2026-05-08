pub enum GuardState {
    Ready(GuardRecord),
    Waiting(String),
}

pub struct GuardRecord {
    label: String,
}

impl GuardRecord {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.trim().to_string(),
        }
    }

    pub fn is_ready(&self) -> bool {
        !self.label.is_empty()
    }

    pub fn render(&self) -> String {
        format!("guard:{}", self.label)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-guard:{}", self.label)
    }
}

pub fn selected_guard(raw: &str) -> String {
    let state = if raw.trim().is_empty() {
        GuardState::Waiting("empty".to_string())
    } else {
        GuardState::Ready(GuardRecord::new(raw))
    };

    match state {
        GuardState::Ready(record) if record.is_ready() => record.render(),
        GuardState::Ready(record) => format!("not-ready:{}", record.render()),
        GuardState::Waiting(label) => format!("waiting:{label}"),
    }
}

pub fn dead_live_guard(raw: &str) -> String {
    GuardRecord::new(raw).dead_method()
}
