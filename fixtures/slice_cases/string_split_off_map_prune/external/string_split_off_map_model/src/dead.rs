pub struct DeadStringSplitOffMapItem {
    value: String,
}

impl DeadStringSplitOffMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-string-split-off-map:{}", self.value)
    }
}

pub fn dead_string_split_off_map(raw: &str) -> String {
    DeadStringSplitOffMapItem::new(raw).dead_method()
}
