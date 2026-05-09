pub struct DeadOptionOrItem {
    value: String,
}

impl DeadOptionOrItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-or:{}", self.value)
    }
}

pub fn dead_option_or(raw: &str) -> String {
    DeadOptionOrItem::new(raw).dead_method()
}
