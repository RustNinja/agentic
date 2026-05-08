pub struct DeadProtocol {
    value: String,
}

impl DeadProtocol {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-protocol:{}", self.value)
    }
}

pub fn dead_protocol(raw: &str) -> String {
    DeadProtocol::new(raw).render()
}
