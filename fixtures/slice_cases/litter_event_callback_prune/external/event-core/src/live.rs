use std::sync::{Arc, RwLock};

use event_protocol::WireEvent;

#[derive(Clone, Copy)]
pub enum EventKind {
    Connected,
    Message,
    Closed,
}

pub struct EventEnvelope {
    label: String,
    payload: String,
    kind: EventKind,
}

impl EventEnvelope {
    pub fn new(label: &str, raw: &str, kind: EventKind) -> Self {
        Self {
            label: normalize_label(label),
            payload: normalize_payload(raw),
            kind,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn payload(&self) -> &str {
        &self.payload
    }

    pub fn kind(&self) -> EventKind {
        self.kind
    }

    pub fn dead_render(&self) -> String {
        format!("dead-envelope:{}", self.label)
    }
}

pub trait EventCallback {
    fn on_event(&self, event: EventEnvelope) -> WireEvent;

    fn dead_trait_method(&self) -> String {
        "dead-event-callback".to_string()
    }
}

pub struct EventBus {
    label: String,
    callback: Arc<RwLock<Option<Arc<dyn EventCallback + Send + Sync>>>>,
}

impl EventBus {
    pub fn new(label: &str) -> Self {
        Self {
            label: normalize_label(label),
            callback: Arc::new(RwLock::new(None)),
        }
    }

    pub fn set_callback(&self, callback: Arc<dyn EventCallback + Send + Sync>) {
        *self
            .callback
            .write()
            .expect("event callback lock should not poison") = Some(callback);
    }

    pub fn emit(&self, raw: &str) -> WireEvent {
        let envelope = EventEnvelope::new(&self.label, raw, classify_payload(raw));
        self.callback
            .read()
            .expect("event callback lock should not poison")
            .as_ref()
            .map(|callback| callback.on_event(envelope))
            .unwrap_or_else(|| WireEvent::missing(&self.label))
    }

    pub fn dead_debug(&self) -> String {
        format!("dead-bus:{}", self.label)
    }
}

fn classify_payload(raw: &str) -> EventKind {
    match raw.trim() {
        "connected" => EventKind::Connected,
        "closed" => EventKind::Closed,
        _ => EventKind::Message,
    }
}

fn normalize_label(label: &str) -> String {
    label.trim().to_ascii_lowercase()
}

fn normalize_payload(raw: &str) -> String {
    raw.trim().replace('\n', " ")
}

pub fn dead_live_event_core(label: &str) -> String {
    EventEnvelope::new(label, "dead", EventKind::Message).dead_render()
}

