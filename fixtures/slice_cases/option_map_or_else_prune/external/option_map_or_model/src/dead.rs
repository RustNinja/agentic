pub struct DeadOptionMapOrItem {
    value: String,
}

impl DeadOptionMapOrItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-map-or:{}", self.value)
    }
}

pub fn dead_option_map_or(raw: &str) -> String {
    DeadOptionMapOrItem::new(raw).dead_method()
}
