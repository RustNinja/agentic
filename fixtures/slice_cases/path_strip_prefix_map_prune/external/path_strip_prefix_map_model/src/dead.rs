pub struct DeadPathStripPrefixMapItem {
    value: String,
}

impl DeadPathStripPrefixMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-path-strip-prefix-map:{}", self.value)
    }
}

pub fn dead_path_strip_prefix_map(raw: &str) -> String {
    DeadPathStripPrefixMapItem::new(raw).dead_method()
}
