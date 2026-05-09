pub struct DeadIteratorFlattenOptionRefsItem {
    value: String,
}

impl DeadIteratorFlattenOptionRefsItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-flatten-option-refs:{}", self.value)
    }
}

pub fn dead_iterator_flatten_option_refs(raw: &str) -> String {
    DeadIteratorFlattenOptionRefsItem::new(raw).dead_method()
}
