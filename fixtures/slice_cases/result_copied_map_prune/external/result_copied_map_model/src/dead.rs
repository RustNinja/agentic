pub struct DeadResultCopiedMapItem {
    value: String,
}

impl DeadResultCopiedMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-result-copied-map:{}", self.value)
    }
}

pub fn dead_result_copied_map(raw: &str) -> String {
    DeadResultCopiedMapItem::new(raw).render()
}
