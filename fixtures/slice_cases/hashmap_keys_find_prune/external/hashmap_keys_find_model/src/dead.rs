pub struct DeadHashmapKeysFindItem {
    value: String,
}

impl DeadHashmapKeysFindItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-keys-find:{}", self.value)
    }
}

pub fn dead_hashmap_keys_find(raw: &str) -> String {
    DeadHashmapKeysFindItem::new(raw).dead_method()
}
