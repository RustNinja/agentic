pub struct DeadHashmapGetMutItem {
    value: String,
}

impl DeadHashmapGetMutItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-get-mut:{}", self.value)
    }
}

pub fn dead_hashmap_get_mut(raw: &str) -> String {
    DeadHashmapGetMutItem::new(raw).dead_method()
}
