pub struct DeadBinaryheapPushPeekMapItem {
    value: String,
}

impl DeadBinaryheapPushPeekMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-binaryheap-push-peek-map:{}", self.value)
    }
}

pub fn dead_binaryheap_push_peek_map(raw: &str) -> String {
    DeadBinaryheapPushPeekMapItem::new(raw).dead_method()
}
