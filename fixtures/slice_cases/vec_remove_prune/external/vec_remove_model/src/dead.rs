pub struct DeadVecRemoveItem {
    value: String,
}

impl DeadVecRemoveItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-remove:{}", self.value)
    }
}

pub fn dead_vec_remove(raw: &str) -> String {
    DeadVecRemoveItem::new(raw).dead_method()
}
