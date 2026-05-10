pub struct DeadStringClearPushStrMapItem {
    value: String,
}

impl DeadStringClearPushStrMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-string-clear-push-str-map:{}", self.value)
    }
}

pub fn dead_string_clear_push_str_map(raw: &str) -> String {
    DeadStringClearPushStrMapItem::new(raw).dead_method()
}
