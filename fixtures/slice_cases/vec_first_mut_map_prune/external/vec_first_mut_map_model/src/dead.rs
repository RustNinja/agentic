pub struct DeadVecFirstMutMapItem {
    value: String,
}

impl DeadVecFirstMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-first-mut-map:{}", self.value)
    }
}

pub fn dead_vec_first_mut_map(raw: &str) -> String {
    DeadVecFirstMutMapItem::new(raw).dead_method()
}
