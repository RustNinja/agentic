pub struct DeadEventBus {
    label: String,
}

impl DeadEventBus {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-core:{}", self.label)
    }
}

pub fn dead_event_core_report(label: &str) -> String {
    DeadEventBus::new(label).render()
}

