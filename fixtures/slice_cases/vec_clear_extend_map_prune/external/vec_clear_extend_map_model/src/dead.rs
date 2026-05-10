pub struct DeadVecClearExtendMapItem {
    value: String,
}

impl DeadVecClearExtendMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-clear-extend-map:{}", self.value)
    }
}

pub fn dead_vec_clear_extend_map(raw: &str) -> String {
    DeadVecClearExtendMapItem::new(raw).dead_method()
}
