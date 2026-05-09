pub struct DeadIteratorFlattenOptionArrayItem {
    value: String,
}

impl DeadIteratorFlattenOptionArrayItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-flatten-option-array:{}", self.value)
    }
}

pub fn dead_iterator_flatten_option_array(raw: &str) -> String {
    DeadIteratorFlattenOptionArrayItem::new(raw).dead_method()
}
