pub struct DeadHashsetDrainItem {
    value: String,
}

impl DeadHashsetDrainItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashset-drain:{}", self.value)
    }
}

pub fn dead_hashset_drain(raw: &str) -> String {
    DeadHashsetDrainItem::new(raw).dead_method()
}
