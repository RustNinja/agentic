pub struct DeadVecRetainMapItem {
    value: String,
}

impl DeadVecRetainMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-retain-map:{}", self.value)
    }
}

pub fn dead_vec_retain_map(raw: &str) -> String {
    DeadVecRetainMapItem::new(raw).dead_method()
}
