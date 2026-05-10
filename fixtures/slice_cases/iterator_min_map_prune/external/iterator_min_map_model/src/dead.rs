pub struct DeadIteratorMinMapItem;

pub struct DeadIteratorMinMapPayload {
    value: String,
}

impl DeadIteratorMinMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-min-map:{}", self.value)
    }
}

pub fn dead_iterator_min_map(raw: &str) -> String {
    DeadIteratorMinMapPayload::new(raw).dead_method()
}
