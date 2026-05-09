pub struct DeadStringReplaceRangeMapItem {
    value: String,
}

impl DeadStringReplaceRangeMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-string-replace-range-map:{}", self.value)
    }
}

pub fn dead_string_replace_range_map(raw: &str) -> String {
    DeadStringReplaceRangeMapItem::new(raw).dead_method()
}
