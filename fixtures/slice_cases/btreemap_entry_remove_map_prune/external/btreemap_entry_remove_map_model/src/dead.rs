pub struct DeadBtreemapEntryRemoveMapItem {
    value: String,
}

impl DeadBtreemapEntryRemoveMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-entry-remove-map:{}", self.value)
    }
}

pub fn dead_btreemap_entry_remove_map(raw: &str) -> String {
    DeadBtreemapEntryRemoveMapItem::new(raw).dead_method()
}
