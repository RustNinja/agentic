pub struct DeadHashmapEntryAndModifyItem {
    value: String,
}

impl DeadHashmapEntryAndModifyItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-entry-and-modify:{}", self.value)
    }
}

pub fn dead_hashmap_entry_and_modify(raw: &str) -> String {
    DeadHashmapEntryAndModifyItem::new(raw).dead_method()
}
