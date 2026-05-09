pub struct DeadHashmapDrainFilterMapItem {
    value: String,
}

impl DeadHashmapDrainFilterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-drain-filter-map:{}", self.value)
    }
}

pub fn dead_hashmap_drain_filter_map(raw: &str) -> String {
    DeadHashmapDrainFilterMapItem::new(raw).dead_method()
}
