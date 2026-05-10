pub struct DeadStrAsBytesFirstMapItem {
    value: String,
}

impl DeadStrAsBytesFirstMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-as-bytes-first-map:{}", self.value)
    }
}

pub fn dead_str_as_bytes_first_map(raw: &str) -> String {
    DeadStrAsBytesFirstMapItem::new(raw).dead_method()
}
