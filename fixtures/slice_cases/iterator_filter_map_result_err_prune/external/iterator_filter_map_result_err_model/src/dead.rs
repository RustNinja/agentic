pub struct DeadIteratorFilterMapResultErrItem {
    value: String,
}

impl DeadIteratorFilterMapResultErrItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-filter-map-result-err:{}", self.value)
    }
}

pub fn dead_iterator_filter_map_result_err(raw: &str) -> String {
    DeadIteratorFilterMapResultErrItem::new(raw).dead_method()
}
