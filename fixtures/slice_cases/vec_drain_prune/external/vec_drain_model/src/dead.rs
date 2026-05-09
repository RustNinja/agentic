pub struct DeadVecDrainItem {
    value: String,
}

impl DeadVecDrainItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-drain:{}", self.value)
    }
}

pub fn dead_vec_drain(raw: &str) -> String {
    DeadVecDrainItem::new(raw).dead_method()
}
