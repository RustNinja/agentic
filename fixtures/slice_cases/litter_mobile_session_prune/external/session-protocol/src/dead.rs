pub struct DeadWireSession {
    value: String,
}

impl DeadWireSession {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-wire-session:{}", self.value)
    }
}

pub fn dead_protocol_report(value: &str) -> String {
    DeadWireSession::new(value).render()
}

