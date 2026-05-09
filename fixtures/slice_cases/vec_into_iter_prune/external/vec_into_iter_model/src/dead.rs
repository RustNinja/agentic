pub struct DeadVecIntoIterItem {
    value: String,
}

impl DeadVecIntoIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-into-iter:{}", self.value)
    }
}

pub fn dead_vec_into_iter(raw: &str) -> String {
    DeadVecIntoIterItem::new(raw).dead_method()
}
