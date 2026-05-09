pub struct DeadVecGetMutMapItem {
    value: String,
}

impl DeadVecGetMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-get-mut-map:{}", self.value)
    }
}

pub fn dead_vec_get_mut_map(raw: &str) -> String {
    DeadVecGetMutMapItem::new(raw).dead_method()
}
