pub struct DeadBinaryheapPeekMutMapItem {
    value: String,
}

impl DeadBinaryheapPeekMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-binaryheap-peek-mut-map:{}", self.value)
    }
}

pub fn dead_binaryheap_peek_mut_map(raw: &str) -> String {
    DeadBinaryheapPeekMutMapItem::new(raw).dead_method()
}
