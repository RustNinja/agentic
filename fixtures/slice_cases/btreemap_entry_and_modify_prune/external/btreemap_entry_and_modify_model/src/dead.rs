pub struct DeadBtreemapEntryAndModifyItem {
    value: String,
}

impl DeadBtreemapEntryAndModifyItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-entry-and-modify:{}", self.value)
    }
}

pub fn dead_btreemap_entry_and_modify(raw: &str) -> String {
    DeadBtreemapEntryAndModifyItem::new(raw).dead_method()
}
