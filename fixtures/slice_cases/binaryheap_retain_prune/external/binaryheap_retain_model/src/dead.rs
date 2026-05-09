pub struct DeadBinaryHeapRetainItem {
    value: String,
}

impl DeadBinaryHeapRetainItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-binaryheap-retain:{}", self.value)
    }
}

pub fn dead_binaryheap_retain(raw: &str) -> String {
    DeadBinaryHeapRetainItem::new(raw).dead_method()
}
