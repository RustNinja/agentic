pub struct DeadStringFromUtf16MapItem {
    value: String,
}

impl DeadStringFromUtf16MapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-string-from-utf16-map:{}", self.value)
    }
}

pub fn dead_string_from_utf16_map(raw: &str) -> String {
    DeadStringFromUtf16MapItem::new(raw).dead_method()
}
