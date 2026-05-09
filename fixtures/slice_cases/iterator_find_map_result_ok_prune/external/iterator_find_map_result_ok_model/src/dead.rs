pub struct DeadIteratorFindMapResultOkItem {
    value: String,
}

impl DeadIteratorFindMapResultOkItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-find-map-result-ok:{}", self.value)
    }
}

pub fn dead_iterator_find_map_result_ok(raw: &str) -> String {
    DeadIteratorFindMapResultOkItem::new(raw).dead_method()
}
