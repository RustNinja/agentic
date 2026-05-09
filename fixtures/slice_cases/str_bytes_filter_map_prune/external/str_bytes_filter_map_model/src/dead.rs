pub struct DeadStrBytesFilterMapItem {
    value: String,
}

impl DeadStrBytesFilterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-bytes-filter-map:{}", self.value)
    }
}

pub fn dead_str_bytes_filter_map(raw: &str) -> String {
    DeadStrBytesFilterMapItem::new(raw).dead_method()
}
