pub struct DeadVecdequeRangeMapItem {
    value: String,
}

impl DeadVecdequeRangeMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-range-map:{}", self.value)
    }
}

pub fn dead_vecdeque_range_map(raw: &str) -> String {
    DeadVecdequeRangeMapItem::new(raw).dead_method()
}
