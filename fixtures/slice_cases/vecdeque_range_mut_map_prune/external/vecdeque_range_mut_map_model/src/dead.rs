pub struct DeadVecdequeRangeMutMapItem;

pub struct DeadVecdequeRangeMutMapPayload {
    value: String,
}

impl DeadVecdequeRangeMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-range-mut-map:{}", self.value)
    }
}

pub fn dead_vecdeque_range_mut_map(raw: &str) -> String {
    DeadVecdequeRangeMutMapPayload::new(raw).dead_method()
}
