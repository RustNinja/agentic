pub struct DeadOptionMapOrValueItem {
    value: String,
}

impl DeadOptionMapOrValueItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-map-or-value:{}", self.value)
    }
}

pub fn dead_option_map_or_value(raw: &str) -> String {
    DeadOptionMapOrValueItem::new(raw).dead_method()
}
