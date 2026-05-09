pub struct DeadBtreemapRangePairsItem {
    value: String,
}

impl DeadBtreemapRangePairsItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-range-pairs:{}", self.value)
    }
}

pub fn dead_btreemap_range_pairs(raw: &str) -> String {
    DeadBtreemapRangePairsItem::new(raw).dead_method()
}
