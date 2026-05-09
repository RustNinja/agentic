pub struct DeadBtreemapEntryOrInsertItem {
    value: String,
}

impl DeadBtreemapEntryOrInsertItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-entry-or-insert:{}", self.value)
    }
}

pub fn dead_btreemap_entry_or_insert(raw: &str) -> String {
    DeadBtreemapEntryOrInsertItem::new(raw).dead_method()
}
