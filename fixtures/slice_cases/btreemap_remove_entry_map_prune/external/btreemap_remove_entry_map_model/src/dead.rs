pub struct DeadBtreemapRemoveEntryMapItem {
    value: String,
}

impl DeadBtreemapRemoveEntryMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-remove-entry-map:{}", self.value)
    }
}

pub fn dead_btreemap_remove_entry_map(raw: &str) -> String {
    DeadBtreemapRemoveEntryMapItem::new(raw).dead_method()
}
