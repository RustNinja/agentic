pub struct DeadVecdequeDrainItem {
    value: String,
}

impl DeadVecdequeDrainItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vecdeque-drain:{}", self.value)
    }
}

pub fn dead_vecdeque_drain(raw: &str) -> String {
    DeadVecdequeDrainItem::new(raw).dead_method()
}
