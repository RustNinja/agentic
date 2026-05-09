pub struct DeadOsstringIntoStringMapItem {
    value: String,
}

impl DeadOsstringIntoStringMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-osstring-into-string-map:{}", self.value)
    }
}

pub fn dead_osstring_into_string_map(raw: &str) -> String {
    DeadOsstringIntoStringMapItem::new(raw).dead_method()
}
