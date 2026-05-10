pub struct DeadReconnectController {
    label: String,
}

impl DeadReconnectController {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-reconnect:{}", self.label)
    }
}

pub fn dead_reconnect(raw: &str) -> String {
    DeadReconnectController::new(raw).render()
}
