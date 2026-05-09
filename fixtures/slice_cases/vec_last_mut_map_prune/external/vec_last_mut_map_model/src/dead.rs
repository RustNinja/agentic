pub struct DeadVecLastMutMapItem {
    value: String,
}

impl DeadVecLastMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-last-mut-map:{}", self.value)
    }
}

pub fn dead_vec_last_mut_map(raw: &str) -> String {
    DeadVecLastMutMapItem::new(raw).dead_method()
}
