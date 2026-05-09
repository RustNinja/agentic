pub struct DeadBinaryheapFromIterPeekMapItem {
    value: String,
}

impl DeadBinaryheapFromIterPeekMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-binaryheap-from-iter-peek-map:{}", self.value)
    }
}

pub fn dead_binaryheap_from_iter_peek_map(raw: &str) -> String {
    DeadBinaryheapFromIterPeekMapItem::new(raw).dead_method()
}
