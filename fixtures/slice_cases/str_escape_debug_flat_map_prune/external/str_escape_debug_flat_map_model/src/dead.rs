pub struct DeadStrEscapeDebugFlatMapItem {
    value: String,
}

impl DeadStrEscapeDebugFlatMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-escape-debug-flat-map:{}", self.value)
    }
}

pub fn dead_str_escape_debug_flat_map(raw: &str) -> String {
    DeadStrEscapeDebugFlatMapItem::new(raw).dead_method()
}
