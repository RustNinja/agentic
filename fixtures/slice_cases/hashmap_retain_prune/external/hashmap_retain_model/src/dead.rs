pub struct DeadHashMapRetainItem {
    value: String,
}

impl DeadHashMapRetainItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-retain:{}", self.value)
    }
}

pub fn dead_hashmap_retain(raw: &str) -> String {
    DeadHashMapRetainItem::new(raw).dead_method()
}
