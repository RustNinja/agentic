pub struct DeadVecExtendFromWithinMapItem {
    value: String,
}

impl DeadVecExtendFromWithinMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-extend-from-within-map:{}", self.value)
    }
}

pub fn dead_vec_extend_from_within_map(raw: &str) -> String {
    DeadVecExtendFromWithinMapItem::new(raw).dead_method()
}
