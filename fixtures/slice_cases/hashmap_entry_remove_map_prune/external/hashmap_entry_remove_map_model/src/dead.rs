pub struct DeadHashmapEntryRemoveMapItem {
    value: String,
}

impl DeadHashmapEntryRemoveMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-entry-remove-map:{}", self.value)
    }
}

pub fn dead_hashmap_entry_remove_map(raw: &str) -> String {
    DeadHashmapEntryRemoveMapItem::new(raw).dead_method()
}
