pub struct DeadVecSplitAtMapItem {
    value: String,
}

impl DeadVecSplitAtMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-split-at-map:{}", self.value)
    }
}

pub fn dead_vec_split_at_map(raw: &str) -> String {
    DeadVecSplitAtMapItem::new(raw).dead_method()
}
