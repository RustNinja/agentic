pub struct DeadIteratorFlattenVecVecItem {
    value: String,
}

impl DeadIteratorFlattenVecVecItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-flatten-vec-vec:{}", self.value)
    }
}

pub fn dead_iterator_flatten_vec_vec(raw: &str) -> String {
    DeadIteratorFlattenVecVecItem::new(raw).dead_method()
}
