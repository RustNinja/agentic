pub struct DeadHashmapKeysMapItem {
    value: String,
}

impl DeadHashmapKeysMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-keys-map:{}", self.value)
    }
}

pub fn dead_hashmap_keys_map(raw: &str) -> String {
    DeadHashmapKeysMapItem::new(raw).dead_method()
}
