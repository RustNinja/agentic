pub struct DeadStrToUppercaseMapItem {
    value: String,
}

impl DeadStrToUppercaseMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-str-to-uppercase-map:{}", self.value)
    }
}

pub fn dead_str_to_uppercase_map(raw: &str) -> String {
    DeadStrToUppercaseMapItem::new(raw).dead_method()
}
