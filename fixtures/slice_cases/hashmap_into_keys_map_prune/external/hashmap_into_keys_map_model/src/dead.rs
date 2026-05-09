pub struct DeadHashmapIntoKeysMapItem {
    value: String,
}

impl DeadHashmapIntoKeysMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-into-keys-map:{}", self.value)
    }
}

pub fn dead_hashmap_into_keys_map(raw: &str) -> String {
    DeadHashmapIntoKeysMapItem::new(raw).dead_method()
}
