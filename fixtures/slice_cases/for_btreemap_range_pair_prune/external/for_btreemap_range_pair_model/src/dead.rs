pub struct DeadForBtreemapRangePairPayload {
    value: String,
}

impl DeadForBtreemapRangePairPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-for-btreemap-range-pair:{}", self.value)
    }
}

pub fn dead_for_btreemap_range_pair(raw: &str) -> String {
    DeadForBtreemapRangePairPayload::new(raw).dead_method()
}
