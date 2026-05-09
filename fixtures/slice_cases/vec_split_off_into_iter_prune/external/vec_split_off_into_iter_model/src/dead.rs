pub struct DeadVecSplitOffIntoIterItem {
    value: String,
}

impl DeadVecSplitOffIntoIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vec-split-off-into-iter:{}", self.value)
    }
}

pub fn dead_vec_split_off_into_iter(raw: &str) -> String {
    DeadVecSplitOffIntoIterItem::new(raw).render()
}
