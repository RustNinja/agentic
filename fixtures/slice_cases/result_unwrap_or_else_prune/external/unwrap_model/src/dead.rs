pub struct DeadUnwrap {
    value: String,
}

impl DeadUnwrap {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-unwrap:{}", self.value)
    }
}

pub fn dead_unwrap(raw: &str) -> String {
    DeadUnwrap::new(raw).render()
}
