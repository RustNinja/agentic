pub struct DeadWireEvent {
    label: String,
}

impl DeadWireEvent {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-protocol:{}", self.label)
    }
}

pub fn dead_event_protocol_report(label: &str) -> String {
    DeadWireEvent::new(label).render()
}

