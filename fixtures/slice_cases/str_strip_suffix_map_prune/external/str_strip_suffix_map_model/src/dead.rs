pub struct DeadStrStripSuffixMapItem {
    value: String,
}

impl DeadStrStripSuffixMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-strip-suffix-map:{}", self.value)
    }
}

pub fn dead_str_strip_suffix_map(raw: &str) -> String {
    DeadStrStripSuffixMapItem::new(raw).dead_method()
}
