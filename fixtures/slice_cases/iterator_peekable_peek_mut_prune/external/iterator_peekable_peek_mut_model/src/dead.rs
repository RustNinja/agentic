pub struct DeadIteratorPeekablePeekMutItem;

pub struct DeadIteratorPeekablePeekMutPayload {
    value: String,
}

impl DeadIteratorPeekablePeekMutPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-peekable-peek-mut:{}", self.value)
    }
}

pub fn dead_iterator_peekable_peek_mut(raw: &str) -> String {
    DeadIteratorPeekablePeekMutPayload::new(raw).dead_method()
}
