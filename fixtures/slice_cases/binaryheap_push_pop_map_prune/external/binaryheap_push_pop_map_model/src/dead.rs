pub struct DeadBinaryheapPushPopMapItem {
    value: String,
}

impl DeadBinaryheapPushPopMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-binaryheap-push-pop-map:{}", self.value)
    }
}

pub fn dead_binaryheap_push_pop_map(raw: &str) -> String {
    DeadBinaryheapPushPopMapItem::new(raw).dead_method()
}
