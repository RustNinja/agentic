pub struct DeadBinaryheapPeekItem {
    value: String,
}

impl DeadBinaryheapPeekItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-binaryheap-peek:{}", self.value)
    }
}

pub fn dead_binaryheap_peek(raw: &str) -> String {
    DeadBinaryheapPeekItem::new(raw).dead_method()
}
