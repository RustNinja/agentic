pub struct DeadBtreemapEntryOrInsertWithItem {
    value: String,
}

impl DeadBtreemapEntryOrInsertWithItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-entry-or-insert-with:{}", self.value)
    }
}

pub fn dead_btreemap_entry_or_insert_with(raw: &str) -> String {
    DeadBtreemapEntryOrInsertWithItem::new(raw).dead_method()
}
