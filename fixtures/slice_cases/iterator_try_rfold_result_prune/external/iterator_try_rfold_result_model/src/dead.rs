pub struct DeadIteratorTryRfoldResultItem;

pub struct DeadIteratorTryRfoldResultPayload {
    value: String,
}

impl DeadIteratorTryRfoldResultPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-try-rfold-result:{}", self.value)
    }
}

pub fn dead_iterator_try_rfold_result(raw: &str) -> String {
    DeadIteratorTryRfoldResultPayload::new(raw).dead_method()
}
