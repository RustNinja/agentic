pub struct DeadIteratorPeekableNextIfItem;

pub struct DeadIteratorPeekableNextIfPayload {
    value: String,
}

impl DeadIteratorPeekableNextIfPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-peekable-next-if:{}", self.value)
    }
}

pub fn dead_iterator_peekable_next_if(raw: &str) -> String {
    DeadIteratorPeekableNextIfPayload::new(raw).dead_method()
}
