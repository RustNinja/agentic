pub struct DeadIteratorForLoopFlattenResultVecItem {
    value: String,
}

impl DeadIteratorForLoopFlattenResultVecItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-for-loop-flatten-result-vec:{}", self.value)
    }
}

pub fn dead_iterator_for_loop_flatten_result_vec(raw: &str) -> String {
    DeadIteratorForLoopFlattenResultVecItem::new(raw).dead_method()
}
