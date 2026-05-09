pub struct DeadBtreemapEntryOrInsertWithKeyItem {
    value: String,
}

impl DeadBtreemapEntryOrInsertWithKeyItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-entry-or-insert-with-key:{}", self.value)
    }
}

pub fn dead_btreemap_entry_or_insert_with_key(raw: &str) -> String {
    DeadBtreemapEntryOrInsertWithKeyItem::new(raw).dead_method()
}
