pub struct DeadOptionUnwrapItem {
    value: String,
}

impl DeadOptionUnwrapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-unwrap:{}", self.value)
    }
}

pub fn dead_option_unwrap(raw: &str) -> String {
    DeadOptionUnwrapItem::new(raw).dead_method()
}
