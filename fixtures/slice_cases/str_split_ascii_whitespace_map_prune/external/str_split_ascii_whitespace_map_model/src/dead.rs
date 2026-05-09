pub struct DeadStrSplitAsciiWhitespaceMapItem {
    value: String,
}

impl DeadStrSplitAsciiWhitespaceMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-split-ascii-whitespace-map:{}", self.value)
    }
}

pub fn dead_str_split_ascii_whitespace_map(raw: &str) -> String {
    DeadStrSplitAsciiWhitespaceMapItem::new(raw).dead_method()
}
