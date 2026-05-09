pub struct DeadHashmapRemoveEntryMapItem {
    value: String,
}

impl DeadHashmapRemoveEntryMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-remove-entry-map:{}", self.value)
    }
}

pub fn dead_hashmap_remove_entry_map(raw: &str) -> String {
    DeadHashmapRemoveEntryMapItem::new(raw).dead_method()
}
