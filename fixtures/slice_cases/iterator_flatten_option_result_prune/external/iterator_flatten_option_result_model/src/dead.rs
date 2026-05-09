pub struct DeadIteratorFlattenOptionResultItem {
    value: String,
}

impl DeadIteratorFlattenOptionResultItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-flatten-option-result:{}", self.value)
    }
}

pub fn dead_iterator_flatten_option_result(raw: &str) -> String {
    DeadIteratorFlattenOptionResultItem::new(raw).dead_method()
}
