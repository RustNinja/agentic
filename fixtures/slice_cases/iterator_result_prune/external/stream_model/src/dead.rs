pub struct DeadEvent {
    value: String,
}

impl DeadEvent {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }
}

pub struct DeadEnvelope {
    events: Vec<DeadEvent>,
}

impl DeadEnvelope {
    pub fn render_dead(self) -> String {
        format!("dead-events:{}", self.events.len())
    }
}

pub fn dead_stream(raw: &str) -> DeadEnvelope {
    DeadEnvelope {
        events: vec![DeadEvent::new(raw)],
    }
}
