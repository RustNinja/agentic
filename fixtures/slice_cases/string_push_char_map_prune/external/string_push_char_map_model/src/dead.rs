pub struct DeadStringPushCharMapItem {
    value: String,
}

impl DeadStringPushCharMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-string-push-char-map:{}", self.value)
    }
}

pub fn dead_string_push_char_map(raw: &str) -> String {
    DeadStringPushCharMapItem::new(raw).dead_method()
}
