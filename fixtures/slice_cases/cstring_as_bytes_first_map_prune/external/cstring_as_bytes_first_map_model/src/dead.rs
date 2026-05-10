pub struct DeadCstringAsBytesFirstMapItem {
    value: String,
}

impl DeadCstringAsBytesFirstMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-cstring-as-bytes-first-map:{}", self.value)
    }
}

pub fn dead_cstring_as_bytes_first_map(raw: &str) -> String {
    DeadCstringAsBytesFirstMapItem::new(raw).dead_method()
}
