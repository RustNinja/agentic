pub struct DeadIteratorPositionItem;

pub struct DeadIteratorPositionPayload {
    value: String,
}

impl DeadIteratorPositionPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-position:{}", self.value)
    }
}

pub fn dead_iterator_position(raw: &str) -> String {
    DeadIteratorPositionPayload::new(raw).dead_method()
}
