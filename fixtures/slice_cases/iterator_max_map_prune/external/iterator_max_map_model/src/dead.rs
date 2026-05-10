pub struct DeadIteratorMaxMapItem;

pub struct DeadIteratorMaxMapPayload {
    value: String,
}

impl DeadIteratorMaxMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-max-map:{}", self.value)
    }
}

pub fn dead_iterator_max_map(raw: &str) -> String {
    DeadIteratorMaxMapPayload::new(raw).dead_method()
}
