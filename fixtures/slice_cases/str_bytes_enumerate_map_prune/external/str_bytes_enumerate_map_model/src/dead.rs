pub struct DeadStrBytesEnumerateMapItem {
    value: String,
}

impl DeadStrBytesEnumerateMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-bytes-enumerate-map:{}", self.value)
    }
}

pub fn dead_str_bytes_enumerate_map(raw: &str) -> String {
    DeadStrBytesEnumerateMapItem::new(raw).dead_method()
}
