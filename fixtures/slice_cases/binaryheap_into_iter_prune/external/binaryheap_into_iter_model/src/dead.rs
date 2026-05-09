pub struct DeadBinaryheapIntoIterItem {
    value: String,
}

impl DeadBinaryheapIntoIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-binaryheap-into-iter:{}", self.value)
    }
}

pub fn dead_binaryheap_into_iter(raw: &str) -> String {
    DeadBinaryheapIntoIterItem::new(raw).dead_method()
}
