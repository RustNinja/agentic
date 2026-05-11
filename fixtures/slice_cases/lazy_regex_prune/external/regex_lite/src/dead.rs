pub struct DeadRegex {
    raw: String,
}

impl DeadRegex {
    pub fn new(raw: &str) -> Self {
        Self {
            raw: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-regex:{}", self.raw)
    }
}

pub fn dead_regex_debug(raw: &str) -> String {
    format!("dead-regex-debug:{raw}")
}
