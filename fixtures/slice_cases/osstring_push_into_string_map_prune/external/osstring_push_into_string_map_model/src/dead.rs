pub struct DeadOsstringPushIntoStringMapItem {
    value: String,
}

impl DeadOsstringPushIntoStringMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-osstring-push-into-string-map:{}", self.value)
    }
}

pub fn dead_osstring_push_into_string_map(raw: &str) -> String {
    DeadOsstringPushIntoStringMapItem::new(raw).dead_method()
}
