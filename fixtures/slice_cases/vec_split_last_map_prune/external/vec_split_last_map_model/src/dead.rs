pub struct DeadVecSplitLastMapItem {
    value: String,
}

impl DeadVecSplitLastMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vec-split-last-map:{}", self.value)
    }
}

pub fn dead_vec_split_last_map(raw: &str) -> String {
    DeadVecSplitLastMapItem::new(raw).render()
}
