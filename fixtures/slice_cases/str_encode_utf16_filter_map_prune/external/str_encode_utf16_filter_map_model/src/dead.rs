pub struct DeadStrEncodeUtf16FilterMapItem {
    value: String,
}

impl DeadStrEncodeUtf16FilterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-encode-utf16-filter-map:{}", self.value)
    }
}

pub fn dead_str_encode_utf16_filter_map(raw: &str) -> String {
    DeadStrEncodeUtf16FilterMapItem::new(raw).dead_method()
}
