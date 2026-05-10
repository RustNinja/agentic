pub struct DeadStringExtendCharsMapItem {
    value: String,
}

impl DeadStringExtendCharsMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-string-extend-chars-map:{}", self.value)
    }
}

pub fn dead_string_extend_chars_map(raw: &str) -> String {
    DeadStringExtendCharsMapItem::new(raw).dead_method()
}
