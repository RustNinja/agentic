pub struct DeadHashmapGetItem {
    value: String,
}

impl DeadHashmapGetItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-get:{}", self.value)
    }
}

pub fn dead_hashmap_get(raw: &str) -> String {
    DeadHashmapGetItem::new(raw).dead_method()
}
