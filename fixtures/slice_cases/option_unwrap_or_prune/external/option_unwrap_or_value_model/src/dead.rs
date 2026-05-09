pub struct DeadOptionUnwrapOrValueItem {
    value: String,
}

impl DeadOptionUnwrapOrValueItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-unwrap-or-value:{}", self.value)
    }
}

pub fn dead_option_unwrap_or_value(raw: &str) -> String {
    DeadOptionUnwrapOrValueItem::new(raw).dead_method()
}
