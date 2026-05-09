pub struct DeadBtreesetReplaceItem {
    value: String,
}

impl DeadBtreesetReplaceItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-replace:{}", self.value)
    }
}

pub fn dead_btreeset_replace(raw: &str) -> String {
    DeadBtreesetReplaceItem::new(raw).dead_method()
}
