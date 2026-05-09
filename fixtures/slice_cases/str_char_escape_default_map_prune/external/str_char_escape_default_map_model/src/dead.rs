pub struct DeadStrCharEscapeDefaultMapItem {
    value: String,
}

impl DeadStrCharEscapeDefaultMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-char-escape-default-map:{}", self.value)
    }
}

pub fn dead_str_char_escape_default_map(raw: &str) -> String {
    DeadStrCharEscapeDefaultMapItem::new(raw).dead_method()
}
