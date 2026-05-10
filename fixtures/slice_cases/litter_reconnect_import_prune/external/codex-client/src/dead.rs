pub struct DeadReconnectState {
    label: String,
}

impl DeadReconnectState {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-reconnect:{}", self.label)
    }
}

pub fn dead_reconnect(raw: &str) -> String {
    DeadReconnectState::new(raw).render()
}
