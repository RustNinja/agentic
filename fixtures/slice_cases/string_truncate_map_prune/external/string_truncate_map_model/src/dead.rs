pub struct DeadStringTruncateMapItem {
    value: String,
}

impl DeadStringTruncateMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-string-truncate-map:{}", self.value)
    }
}

pub fn dead_string_truncate_map(raw: &str) -> String {
    DeadStringTruncateMapItem::new(raw).dead_method()
}
