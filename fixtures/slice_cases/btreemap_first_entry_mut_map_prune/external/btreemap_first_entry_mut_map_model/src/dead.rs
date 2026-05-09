pub struct DeadBtreemapFirstEntryMutMapItem {
    value: String,
}

impl DeadBtreemapFirstEntryMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-first-entry-mut-map:{}", self.value)
    }
}

pub fn dead_btreemap_first_entry_mut_map(raw: &str) -> String {
    DeadBtreemapFirstEntryMutMapItem::new(raw).dead_method()
}
