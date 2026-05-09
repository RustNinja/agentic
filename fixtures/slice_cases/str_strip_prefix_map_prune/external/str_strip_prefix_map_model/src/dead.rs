pub struct DeadStrStripPrefixMapItem {
    value: String,
}

impl DeadStrStripPrefixMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-strip-prefix-map:{}", self.value)
    }
}

pub fn dead_str_strip_prefix_map(raw: &str) -> String {
    DeadStrStripPrefixMapItem::new(raw).dead_method()
}
