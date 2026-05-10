pub struct DeadStringRetainMapItem {
    value: String,
}

impl DeadStringRetainMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-string-retain-map:{}", self.value)
    }
}

pub fn dead_string_retain_map(raw: &str) -> String {
    DeadStringRetainMapItem::new(raw).dead_method()
}
