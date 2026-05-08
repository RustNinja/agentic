pub struct DeadDedupItem {
    value: String,
}

impl DeadDedupItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-dedup:{}", self.value)
    }
}

pub fn dead_dedup_by(raw: &str) -> String {
    DeadDedupItem::new(raw).render()
}
