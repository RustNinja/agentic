pub struct DeadVecSplitFirstMapItem {
    value: String,
}

impl DeadVecSplitFirstMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vec-split-first-map:{}", self.value)
    }
}

pub fn dead_vec_split_first_map(raw: &str) -> String {
    DeadVecSplitFirstMapItem::new(raw).render()
}
