pub struct DeadForBtreemapRangeMutPairPayload {
    value: String,
}

impl DeadForBtreemapRangeMutPairPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-for-btreemap-range-mut-pair:{}", self.value)
    }
}

pub fn dead_for_btreemap_range_mut_pair(raw: &str) -> String {
    DeadForBtreemapRangeMutPairPayload::new(raw).dead_method()
}
