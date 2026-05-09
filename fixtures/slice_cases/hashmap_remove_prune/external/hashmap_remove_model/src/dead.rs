pub struct DeadHashmapRemoveItem {
    value: String,
}

impl DeadHashmapRemoveItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-remove:{}", self.value)
    }
}

pub fn dead_hashmap_remove(raw: &str) -> String {
    DeadHashmapRemoveItem::new(raw).dead_method()
}
