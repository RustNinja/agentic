pub struct DeadIteratorFlattenTakeMapItem {
    value: String,
}

impl DeadIteratorFlattenTakeMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-flatten-take-map:{}", self.value)
    }
}

pub fn dead_iterator_flatten_take_map(raw: &str) -> String {
    DeadIteratorFlattenTakeMapItem::new(raw).dead_method()
}
