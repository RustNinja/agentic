pub struct DeadVecDedupByKeyItem {
    value: String,
}

impl DeadVecDedupByKeyItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-dedup-by-key:{}", self.value)
    }
}

pub fn dead_vec_dedup_by_key(raw: &str) -> String {
    DeadVecDedupByKeyItem::new(raw).dead_method()
}
