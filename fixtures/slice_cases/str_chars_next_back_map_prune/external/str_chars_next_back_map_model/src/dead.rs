pub struct DeadStrCharsNextBackMapItem {
    value: String,
}

impl DeadStrCharsNextBackMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-chars-next-back-map:{}", self.value)
    }
}

pub fn dead_str_chars_next_back_map(raw: &str) -> String {
    DeadStrCharsNextBackMapItem::new(raw).dead_method()
}
