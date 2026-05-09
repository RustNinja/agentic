pub struct DeadBtreemapEntryOrDefaultItem {
    value: String,
}

impl DeadBtreemapEntryOrDefaultItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-entry-or-default:{}", self.value)
    }
}

pub fn dead_btreemap_entry_or_default(raw: &str) -> String {
    DeadBtreemapEntryOrDefaultItem::new(raw).dead_method()
}
