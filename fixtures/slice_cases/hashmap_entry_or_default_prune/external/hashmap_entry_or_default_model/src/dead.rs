pub struct DeadHashmapEntryOrDefaultItem {
    value: String,
}

impl DeadHashmapEntryOrDefaultItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-entry-or-default:{}", self.value)
    }
}

pub fn dead_hashmap_entry_or_default(raw: &str) -> String {
    DeadHashmapEntryOrDefaultItem::new(raw).dead_method()
}
