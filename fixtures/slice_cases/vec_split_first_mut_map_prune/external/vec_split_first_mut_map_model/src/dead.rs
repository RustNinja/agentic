pub struct DeadVecSplitFirstMutMapItem {
    value: String,
}

impl DeadVecSplitFirstMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vec-split-first-mut-map:{}", self.value)
    }
}

pub fn dead_vec_split_first_mut_map(raw: &str) -> String {
    DeadVecSplitFirstMutMapItem::new(raw).render()
}
