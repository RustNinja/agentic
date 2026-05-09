pub struct DeadResultMatchItem {
    value: String,
}

impl DeadResultMatchItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-result-match:{}", self.value)
    }
}

pub fn dead_result_match(raw: &str) -> String {
    DeadResultMatchItem::new(raw).render()
}
