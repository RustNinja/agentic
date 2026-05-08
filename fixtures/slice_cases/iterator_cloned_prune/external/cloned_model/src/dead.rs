pub struct DeadClonedItem {
    value: String,
}

impl DeadClonedItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-cloned:{}", self.value)
    }
}

pub fn dead_cloned(raw: &str) -> String {
    DeadClonedItem::new(raw).render()
}
