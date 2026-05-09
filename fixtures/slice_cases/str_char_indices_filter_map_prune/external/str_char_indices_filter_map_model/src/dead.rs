pub struct DeadStrCharIndicesFilterMapItem {
    value: String,
}

impl DeadStrCharIndicesFilterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-char-indices-filter-map:{}", self.value)
    }
}

pub fn dead_str_char_indices_filter_map(raw: &str) -> String {
    DeadStrCharIndicesFilterMapItem::new(raw).dead_method()
}
