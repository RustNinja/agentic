pub struct DeadBtreemapLastEntryMutMapItem {
    value: String,
}

impl DeadBtreemapLastEntryMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-last-entry-mut-map:{}", self.value)
    }
}

pub fn dead_btreemap_last_entry_mut_map(raw: &str) -> String {
    DeadBtreemapLastEntryMutMapItem::new(raw).dead_method()
}
