pub struct DeadIteratorPeekablePeekMapItem {
    value: String,
}

impl DeadIteratorPeekablePeekMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-peekable-peek-map:{}", self.value)
    }
}

pub fn dead_iterator_peekable_peek_map(raw: &str) -> String {
    DeadIteratorPeekablePeekMapItem::new(raw).dead_method()
}
