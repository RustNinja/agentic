pub struct DeadHashmapEntryOrInsertWithItem {
    value: String,
}

impl DeadHashmapEntryOrInsertWithItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-entry-or-insert-with:{}", self.value)
    }
}

pub fn dead_hashmap_entry_or_insert_with(raw: &str) -> String {
    DeadHashmapEntryOrInsertWithItem::new(raw).dead_method()
}
