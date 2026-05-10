pub struct DeadIteratorNextBackMapItem;

pub struct DeadIteratorNextBackMapPayload {
    value: String,
}

impl DeadIteratorNextBackMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-next-back-map:{}", self.value)
    }
}

pub fn dead_iterator_next_back_map(raw: &str) -> String {
    DeadIteratorNextBackMapPayload::new(raw).dead_method()
}
