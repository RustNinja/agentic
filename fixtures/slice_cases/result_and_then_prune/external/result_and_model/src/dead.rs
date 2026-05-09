pub struct DeadResultAndItem {
    value: String,
}

impl DeadResultAndItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-result-and:{}", self.value)
    }
}

pub fn dead_result_and(raw: &str) -> String {
    DeadResultAndItem::new(raw).render()
}
