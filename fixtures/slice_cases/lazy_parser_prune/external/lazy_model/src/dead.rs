pub struct DeadLazyParser {
    prefix: String,
}

impl DeadLazyParser {
    pub fn new(raw: &str) -> Self {
        Self {
            prefix: raw.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-parser:{}", self.prefix)
    }
}

pub fn dead_lazy(raw: &str) -> String {
    DeadLazyParser::new(raw).render()
}
