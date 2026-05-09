pub struct DeadIteratorForLoopFlattenOptionRefsItem {
    value: String,
}

impl DeadIteratorForLoopFlattenOptionRefsItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-for-loop-flatten-option-refs:{}", self.value)
    }
}

pub fn dead_iterator_for_loop_flatten_option_refs(raw: &str) -> String {
    DeadIteratorForLoopFlattenOptionRefsItem::new(raw).dead_method()
}
