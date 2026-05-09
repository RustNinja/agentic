pub struct DeadHashmapEntryMatchItem {
    value: String,
}

impl DeadHashmapEntryMatchItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-entry-match:{}", self.value)
    }
}

pub fn dead_hashmap_entry_match(raw: &str) -> String {
    DeadHashmapEntryMatchItem::new(raw).dead_method()
}
