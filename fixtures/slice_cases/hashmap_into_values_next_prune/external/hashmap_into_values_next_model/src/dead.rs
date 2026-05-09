pub struct DeadHashmapIntoValuesNextItem {
    value: String,
}

impl DeadHashmapIntoValuesNextItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-into-values-next:{}", self.value)
    }
}

pub fn dead_hashmap_into_values_next(raw: &str) -> String {
    DeadHashmapIntoValuesNextItem::new(raw).dead_method()
}
