pub struct DeadBinaryheapIterFindItem {
    value: String,
}

impl DeadBinaryheapIterFindItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-binaryheap-iter-find:{}", self.value)
    }
}

pub fn dead_binaryheap_iter_find(raw: &str) -> String {
    DeadBinaryheapIterFindItem::new(raw).dead_method()
}
