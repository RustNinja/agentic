#[derive(Clone, Copy)]
pub enum WireEventKind {
    Connected,
    Message,
    Closed,
    Missing,
}

pub struct WireEvent {
    label: String,
    body: String,
    kind: WireEventKind,
}

impl WireEvent {
    pub fn connected(label: &str, body: &str) -> Self {
        Self::new(label, body, WireEventKind::Connected)
    }

    pub fn message(label: &str, body: &str) -> Self {
        Self::new(label, body, WireEventKind::Message)
    }

    pub fn closed(label: &str, body: &str) -> Self {
        Self::new(label, body, WireEventKind::Closed)
    }

    pub fn missing(label: &str) -> Self {
        Self::new(label, "missing", WireEventKind::Missing)
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn kind(&self) -> WireEventKind {
        self.kind
    }

    fn new(label: &str, body: &str, kind: WireEventKind) -> Self {
        Self {
            label: label.to_string(),
            body: body.to_string(),
            kind,
        }
    }

    pub fn dead_summary(&self) -> String {
        format!("dead-wire:{}", self.label)
    }
}

pub fn dead_live_event_protocol() -> WireEvent {
    WireEvent::new("dead", "dead", WireEventKind::Message)
}

