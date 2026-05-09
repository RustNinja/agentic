pub struct DeadBtreesetRangeFindItem {
    value: String,
}

impl DeadBtreesetRangeFindItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-range-find:{}", self.value)
    }
}

pub fn dead_btreeset_range_find(raw: &str) -> String {
    DeadBtreesetRangeFindItem::new(raw).dead_method()
}
