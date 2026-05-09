pub struct DeadVecFirstItem {
    value: String,
}

impl DeadVecFirstItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-first:{}", self.value)
    }
}

pub fn dead_vec_first(raw: &str) -> String {
    DeadVecFirstItem::new(raw).dead_method()
}
