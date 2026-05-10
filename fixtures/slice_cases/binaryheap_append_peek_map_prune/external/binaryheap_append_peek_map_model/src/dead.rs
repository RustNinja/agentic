pub struct DeadBinaryheapAppendPeekMapItem {
    value: String,
}

impl DeadBinaryheapAppendPeekMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-binaryheap-append-peek-map:{}", self.value)
    }
}

pub fn dead_binaryheap_append_peek_map(raw: &str) -> String {
    DeadBinaryheapAppendPeekMapItem::new(raw).dead_method()
}
