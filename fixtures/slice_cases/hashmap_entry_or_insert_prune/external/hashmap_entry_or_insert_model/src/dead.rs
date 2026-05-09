pub struct DeadHashmapEntryOrInsertItem {
    value: String,
}

impl DeadHashmapEntryOrInsertItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-entry-or-insert:{}", self.value)
    }
}

pub fn dead_hashmap_entry_or_insert(raw: &str) -> String {
    DeadHashmapEntryOrInsertItem::new(raw).dead_method()
}
