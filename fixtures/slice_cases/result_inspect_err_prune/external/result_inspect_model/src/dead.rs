pub struct DeadResultInspectItem {
    value: String,
}

impl DeadResultInspectItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-result-inspect:{}", self.value)
    }
}

pub fn dead_result_inspect(raw: &str) -> String {
    DeadResultInspectItem::new(raw).render()
}
