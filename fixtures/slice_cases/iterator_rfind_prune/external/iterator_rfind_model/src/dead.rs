pub struct DeadIteratorRfindItem;

pub struct DeadIteratorRfindPayload {
    value: String,
}

impl DeadIteratorRfindPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-rfind:{}", self.value)
    }
}

pub fn dead_iterator_rfind(raw: &str) -> String {
    DeadIteratorRfindPayload::new(raw).dead_method()
}
