pub struct ProtocolPayload {
    value: String,
}

impl ProtocolPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("payload:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-payload:{}", self.value)
    }
}

enum ProtocolEvent {
    Accepted { payload: ProtocolPayload },
    Rejected,
    Ignored(ProtocolPayload),
}

fn decode_event(raw: &str) -> ProtocolEvent {
    if raw.trim().is_empty() {
        ProtocolEvent::Rejected
    } else {
        ProtocolEvent::Accepted {
            payload: ProtocolPayload::new(raw),
        }
    }
}

pub fn selected_protocol(raw: &str) -> String {
    match decode_event(raw) {
        ProtocolEvent::Accepted { payload } => payload.render(),
        ProtocolEvent::Rejected => "rejected".to_string(),
        _ => "ignored".to_string(),
    }
}

pub fn dead_live_protocol(raw: &str) -> String {
    ProtocolPayload::new(raw).dead_method()
}
