pub struct DeadHashmapValuesFilterMapItem {
    value: String,
}

impl DeadHashmapValuesFilterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-values-filter-map:{}", self.value)
    }
}

pub fn dead_hashmap_values_filter_map(raw: &str) -> String {
    DeadHashmapValuesFilterMapItem::new(raw).dead_method()
}
