pub struct DeadOsstringClearPushIntoStringMapItem {
    value: String,
}

impl DeadOsstringClearPushIntoStringMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-osstring-clear-push-into-string-map:{}", self.value)
    }
}

pub fn dead_osstring_clear_push_into_string_map(raw: &str) -> String {
    DeadOsstringClearPushIntoStringMapItem::new(raw).dead_method()
}
