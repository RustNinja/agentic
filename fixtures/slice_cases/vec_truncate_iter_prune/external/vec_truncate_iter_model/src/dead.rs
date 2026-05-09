pub struct DeadVecTruncateIterItem {
    value: String,
}

impl DeadVecTruncateIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vec-truncate-iter:{}", self.value)
    }
}

pub fn dead_vec_truncate_iter(raw: &str) -> String {
    DeadVecTruncateIterItem::new(raw).render()
}
