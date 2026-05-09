pub struct DeadVecIterFindItem {
    value: String,
}

impl DeadVecIterFindItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-iter-find:{}", self.value)
    }
}

pub fn dead_vec_iter_find(raw: &str) -> String {
    DeadVecIterFindItem::new(raw).dead_method()
}
