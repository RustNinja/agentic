pub struct DeadIteratorRfoldItem;

pub struct DeadIteratorRfoldPayload {
    value: String,
}

impl DeadIteratorRfoldPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-rfold:{}", self.value)
    }
}

pub fn dead_iterator_rfold(raw: &str) -> String {
    DeadIteratorRfoldPayload::new(raw).dead_method()
}
