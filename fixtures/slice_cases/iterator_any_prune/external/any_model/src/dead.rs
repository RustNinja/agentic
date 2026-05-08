pub struct DeadAnyFlag {
    value: String,
}

impl DeadAnyFlag {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-any-flag:{}", self.value)
    }
}

pub fn dead_any(raw: &str) -> String {
    DeadAnyFlag::new(raw).render()
}
