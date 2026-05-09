pub struct DeadOptionMapItem {
    value: String,
}

impl DeadOptionMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-map:{}", self.value)
    }
}

pub fn dead_option_map(raw: &str) -> String {
    DeadOptionMapItem::new(raw).dead_method()
}
