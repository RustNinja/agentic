pub struct DeadVecDedupByMapItem {
    value: String,
}

impl DeadVecDedupByMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-dedup-by-map:{}", self.value)
    }
}

pub fn dead_vec_dedup_by_map(raw: &str) -> String {
    DeadVecDedupByMapItem::new(raw).dead_method()
}
