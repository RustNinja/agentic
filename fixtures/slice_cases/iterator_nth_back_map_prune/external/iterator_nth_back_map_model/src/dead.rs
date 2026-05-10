pub struct DeadIteratorNthBackMapItem;

pub struct DeadIteratorNthBackMapPayload {
    value: String,
}

impl DeadIteratorNthBackMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-nth-back-map:{}", self.value)
    }
}

pub fn dead_iterator_nth_back_map(raw: &str) -> String {
    DeadIteratorNthBackMapPayload::new(raw).dead_method()
}
