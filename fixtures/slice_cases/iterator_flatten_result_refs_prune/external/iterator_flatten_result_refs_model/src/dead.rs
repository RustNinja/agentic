pub struct DeadIteratorFlattenResultRefsItem {
    value: String,
}

impl DeadIteratorFlattenResultRefsItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-flatten-result-refs:{}", self.value)
    }
}

pub fn dead_iterator_flatten_result_refs(raw: &str) -> String {
    DeadIteratorFlattenResultRefsItem::new(raw).dead_method()
}
