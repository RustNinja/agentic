pub struct DeadEntryMapItem {
    value: String,
}

impl DeadEntryMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-entry-map:{}", self.value)
    }
}

pub fn dead_entry_map(raw: &str) -> String {
    DeadEntryMapItem::new(raw).render()
}
