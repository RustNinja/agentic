pub struct DeadHashmapKeysClonedMapItem {
    value: String,
}

impl DeadHashmapKeysClonedMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-keys-cloned-map:{}", self.value)
    }
}

pub fn dead_hashmap_keys_cloned_map(raw: &str) -> String {
    DeadHashmapKeysClonedMapItem::new(raw).dead_method()
}
