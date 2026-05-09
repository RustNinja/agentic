pub struct DeadIteratorFlattenResultVecItem {
    value: String,
}

impl DeadIteratorFlattenResultVecItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-flatten-result-vec:{}", self.value)
    }
}

pub fn dead_iterator_flatten_result_vec(raw: &str) -> String {
    DeadIteratorFlattenResultVecItem::new(raw).dead_method()
}
