pub struct DeadVecDrainFilterMapItem {
    value: String,
}

impl DeadVecDrainFilterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-drain-filter-map:{}", self.value)
    }
}

pub fn dead_vec_drain_filter_map(raw: &str) -> String {
    DeadVecDrainFilterMapItem::new(raw).dead_method()
}
