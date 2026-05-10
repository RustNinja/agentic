pub struct DeadVecSplitOffLastMapItem {
    value: String,
}

impl DeadVecSplitOffLastMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-split-off-last-map:{}", self.value)
    }
}

pub fn dead_vec_split_off_last_map(raw: &str) -> String {
    DeadVecSplitOffLastMapItem::new(raw).dead_method()
}
