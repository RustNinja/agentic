pub struct DeadVecTruncateGetMapItem {
    value: String,
}

impl DeadVecTruncateGetMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-truncate-get-map:{}", self.value)
    }
}

pub fn dead_vec_truncate_get_map(raw: &str) -> String {
    DeadVecTruncateGetMapItem::new(raw).dead_method()
}
