pub struct DeadVecRetainMutItem {
    value: String,
}

impl DeadVecRetainMutItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-retain-mut:{}", self.value)
    }
}

pub fn dead_vec_retain_mut(raw: &str) -> String {
    DeadVecRetainMutItem::new(raw).dead_method()
}
