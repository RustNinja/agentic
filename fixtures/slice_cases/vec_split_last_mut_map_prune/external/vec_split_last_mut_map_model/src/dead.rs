pub struct DeadVecSplitLastMutMapItem {
    value: String,
}

impl DeadVecSplitLastMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vec-split-last-mut-map:{}", self.value)
    }
}

pub fn dead_vec_split_last_mut_map(raw: &str) -> String {
    DeadVecSplitLastMutMapItem::new(raw).render()
}
