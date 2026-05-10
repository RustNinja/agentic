pub struct DeadVecSplitAtMutMapItem {
    value: String,
}

impl DeadVecSplitAtMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-split-at-mut-map:{}", self.value)
    }
}

pub fn dead_vec_split_at_mut_map(raw: &str) -> String {
    DeadVecSplitAtMutMapItem::new(raw).dead_method()
}
