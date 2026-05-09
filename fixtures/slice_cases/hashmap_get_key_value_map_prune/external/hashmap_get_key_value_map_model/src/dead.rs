pub struct DeadHashmapGetKeyValueMapItem {
    value: String,
}

impl DeadHashmapGetKeyValueMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-get-key-value-map:{}", self.value)
    }
}

pub fn dead_hashmap_get_key_value_map(raw: &str) -> String {
    DeadHashmapGetKeyValueMapItem::new(raw).dead_method()
}
