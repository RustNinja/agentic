pub struct DeadHashmapValuesNextItem {
    value: String,
}

impl DeadHashmapValuesNextItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-values-next:{}", self.value)
    }
}

pub fn dead_hashmap_values_next(raw: &str) -> String {
    DeadHashmapValuesNextItem::new(raw).dead_method()
}
