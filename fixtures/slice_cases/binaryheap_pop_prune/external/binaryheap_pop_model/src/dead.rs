pub struct DeadBinaryheapPopItem {
    value: String,
}

impl DeadBinaryheapPopItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-binaryheap-pop:{}", self.value)
    }
}

pub fn dead_binaryheap_pop(raw: &str) -> String {
    DeadBinaryheapPopItem::new(raw).dead_method()
}
