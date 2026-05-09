pub struct DeadHashSetRetainItem {
    value: String,
}

impl DeadHashSetRetainItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashset-retain:{}", self.value)
    }
}

pub fn dead_hashset_retain(raw: &str) -> String {
    DeadHashSetRetainItem::new(raw).dead_method()
}
