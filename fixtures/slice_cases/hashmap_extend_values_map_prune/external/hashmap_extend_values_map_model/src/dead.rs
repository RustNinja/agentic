pub struct DeadHashmapExtendValuesMapItem {
    value: String,
}

impl DeadHashmapExtendValuesMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-extend-values-map:{}", self.value)
    }
}

pub fn dead_hashmap_extend_values_map(raw: &str) -> String {
    DeadHashmapExtendValuesMapItem::new(raw).dead_method()
}
