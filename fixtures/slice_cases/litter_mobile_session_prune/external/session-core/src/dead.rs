pub struct DeadSessionEngine {
    value: String,
}

impl DeadSessionEngine {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-session-engine:{}", self.value)
    }
}

pub fn dead_core_report(value: &str) -> String {
    DeadSessionEngine::new(value).render()
}

