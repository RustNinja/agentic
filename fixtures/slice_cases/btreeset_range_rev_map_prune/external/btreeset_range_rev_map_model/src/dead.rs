pub struct DeadBtreesetRangeRevMapItem {
    value: String,
}

impl DeadBtreesetRangeRevMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-range-rev-map:{}", self.value)
    }
}

pub fn dead_btreeset_range_rev_map(raw: &str) -> String {
    DeadBtreesetRangeRevMapItem::new(raw).dead_method()
}
