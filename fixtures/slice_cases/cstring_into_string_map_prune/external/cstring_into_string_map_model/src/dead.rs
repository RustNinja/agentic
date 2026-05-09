pub struct DeadCstringIntoStringMapItem {
    value: String,
}

impl DeadCstringIntoStringMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-cstring-into-string-map:{}", self.value)
    }
}

pub fn dead_cstring_into_string_map(raw: &str) -> String {
    DeadCstringIntoStringMapItem::new(raw).dead_method()
}
