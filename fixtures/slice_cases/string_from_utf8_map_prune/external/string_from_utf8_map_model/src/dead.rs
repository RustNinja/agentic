pub struct DeadStringFromUtf8MapItem {
    value: String,
}

impl DeadStringFromUtf8MapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-string-from-utf8-map:{}", self.value)
    }
}

pub fn dead_string_from_utf8_map(raw: &str) -> String {
    DeadStringFromUtf8MapItem::new(raw).dead_method()
}
