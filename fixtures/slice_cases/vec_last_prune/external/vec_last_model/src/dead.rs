pub struct DeadVecLastItem {
    value: String,
}

impl DeadVecLastItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-last:{}", self.value)
    }
}

pub fn dead_vec_last(raw: &str) -> String {
    DeadVecLastItem::new(raw).dead_method()
}
