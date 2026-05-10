pub struct DeadStrToLowercaseMapItem {
    value: String,
}

impl DeadStrToLowercaseMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-to-lowercase-map:{}", self.value)
    }
}

pub fn dead_str_to_lowercase_map(raw: &str) -> String {
    DeadStrToLowercaseMapItem::new(raw).dead_method()
}
