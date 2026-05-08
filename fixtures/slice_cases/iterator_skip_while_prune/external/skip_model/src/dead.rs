pub struct DeadSkipItem {
    value: String,
}

impl DeadSkipItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-skip:{}", self.value)
    }
}

pub fn dead_skip_while(raw: &str) -> String {
    DeadSkipItem::new(raw).render()
}
