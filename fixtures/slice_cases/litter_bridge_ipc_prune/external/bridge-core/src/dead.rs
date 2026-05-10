pub struct DeadBridgeSession {
    label: String,
}

impl DeadBridgeSession {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-core:{}", self.label)
    }
}

pub fn dead_core_report(label: &str) -> String {
    DeadBridgeSession::new(label).render()
}

