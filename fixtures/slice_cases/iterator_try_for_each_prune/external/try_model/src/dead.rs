pub struct DeadTryStep {
    value: String,
}

impl DeadTryStep {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-try-step:{}", self.value)
    }
}

pub fn dead_try_for_each(raw: &str) -> String {
    DeadTryStep::new(raw).render()
}
