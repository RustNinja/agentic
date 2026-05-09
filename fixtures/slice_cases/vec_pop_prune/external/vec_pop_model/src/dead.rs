pub struct DeadVecPopItem {
    value: String,
}

impl DeadVecPopItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-pop:{}", self.value)
    }
}

pub fn dead_vec_pop(raw: &str) -> String {
    DeadVecPopItem::new(raw).dead_method()
}
