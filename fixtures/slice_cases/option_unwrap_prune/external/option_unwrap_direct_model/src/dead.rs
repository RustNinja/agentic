pub struct DeadOptionUnwrapDirectItem {
    value: String,
}

impl DeadOptionUnwrapDirectItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-unwrap-direct:{}", self.value)
    }
}

pub fn dead_option_unwrap_direct(raw: &str) -> String {
    DeadOptionUnwrapDirectItem::new(raw).dead_method()
}
