pub struct DeadHashmapEntryOrInsertWithKeyItem {
    value: String,
}

impl DeadHashmapEntryOrInsertWithKeyItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-entry-or-insert-with-key:{}", self.value)
    }
}

pub fn dead_hashmap_entry_or_insert_with_key(raw: &str) -> String {
    DeadHashmapEntryOrInsertWithKeyItem::new(raw).dead_method()
}
