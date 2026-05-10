pub struct DeadStringIntoBytesFirstMapItem {
    value: String,
}

impl DeadStringIntoBytesFirstMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-string-into-bytes-first-map:{}", self.value)
    }
}

pub fn dead_string_into_bytes_first_map(raw: &str) -> String {
    DeadStringIntoBytesFirstMapItem::new(raw).dead_method()
}
