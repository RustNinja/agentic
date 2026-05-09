pub struct DeadStringPopCharMapItem {
    value: String,
}

impl DeadStringPopCharMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-string-pop-char-map:{}", self.value)
    }
}

pub fn dead_string_pop_char_map(raw: &str) -> String {
    DeadStringPopCharMapItem::new(raw).dead_method()
}
