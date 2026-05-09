pub struct DeadIteratorFilterMapResultOkItem {
    value: String,
}

impl DeadIteratorFilterMapResultOkItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-filter-map-result-ok:{}", self.value)
    }
}

pub fn dead_iterator_filter_map_result_ok(raw: &str) -> String {
    DeadIteratorFilterMapResultOkItem::new(raw).dead_method()
}
