pub struct DeadStrCharsFilterMapItem {
    value: String,
}

impl DeadStrCharsFilterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-chars-filter-map:{}", self.value)
    }
}

pub fn dead_str_chars_filter_map(raw: &str) -> String {
    DeadStrCharsFilterMapItem::new(raw).dead_method()
}
