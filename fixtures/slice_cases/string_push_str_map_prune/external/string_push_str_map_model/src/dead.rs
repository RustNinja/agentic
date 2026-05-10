pub struct DeadStringPushStrMapItem {
    value: String,
}

impl DeadStringPushStrMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-string-push-str-map:{}", self.value)
    }
}

pub fn dead_string_push_str_map(raw: &str) -> String {
    DeadStringPushStrMapItem::new(raw).dead_method()
}
