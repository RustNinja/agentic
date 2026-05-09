pub struct DeadBinaryheapDrainItem {
    value: String,
}

impl DeadBinaryheapDrainItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-binaryheap-drain:{}", self.value)
    }
}

pub fn dead_binaryheap_drain(raw: &str) -> String {
    DeadBinaryheapDrainItem::new(raw).dead_method()
}
