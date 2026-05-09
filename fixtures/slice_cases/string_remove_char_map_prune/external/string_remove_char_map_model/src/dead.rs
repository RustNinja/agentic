pub struct DeadStringRemoveCharMapItem {
    value: String,
}

impl DeadStringRemoveCharMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-string-remove-char-map:{}", self.value)
    }
}

pub fn dead_string_remove_char_map(raw: &str) -> String {
    DeadStringRemoveCharMapItem::new(raw).dead_method()
}
