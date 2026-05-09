pub struct DeadVecGetItem {
    value: String,
}

impl DeadVecGetItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-get:{}", self.value)
    }
}

pub fn dead_vec_get(raw: &str) -> String {
    DeadVecGetItem::new(raw).dead_method()
}
