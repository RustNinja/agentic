pub struct DeadResultOrItem {
    value: String,
}

impl DeadResultOrItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-result-or:{}", self.value)
    }
}

pub fn dead_result_or(raw: &str) -> String {
    DeadResultOrItem::new(raw).render()
}
