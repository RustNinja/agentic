pub struct DeadVecInsertGetMapItem {
    value: String,
}

impl DeadVecInsertGetMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-insert-get-map:{}", self.value)
    }
}

pub fn dead_vec_insert_get_map(raw: &str) -> String {
    DeadVecInsertGetMapItem::new(raw).dead_method()
}
