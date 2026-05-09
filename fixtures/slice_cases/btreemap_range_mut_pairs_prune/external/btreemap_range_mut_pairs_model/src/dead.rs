pub struct DeadBtreemapRangeMutPairsItem {
    value: String,
}

impl DeadBtreemapRangeMutPairsItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-range-mut-pairs:{}", self.value)
    }
}

pub fn dead_btreemap_range_mut_pairs(raw: &str) -> String {
    DeadBtreemapRangeMutPairsItem::new(raw).dead_method()
}
