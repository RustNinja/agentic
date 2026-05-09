pub struct DeadVecSpliceMapItem {
    value: String,
}

impl DeadVecSpliceMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vec-splice-map:{}", self.value)
    }
}

pub fn dead_vec_splice_map(raw: &str) -> String {
    DeadVecSpliceMapItem::new(raw).render()
}
