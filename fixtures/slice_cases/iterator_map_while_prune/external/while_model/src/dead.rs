pub struct DeadWhileStep {
    value: String,
}

impl DeadWhileStep {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-while-step:{}", self.value)
    }
}

pub fn dead_map_while(raw: &str) -> String {
    DeadWhileStep::new(raw).render()
}
