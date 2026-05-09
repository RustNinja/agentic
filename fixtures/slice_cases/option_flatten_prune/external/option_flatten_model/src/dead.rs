pub struct DeadOptionFlattenItem {
    value: String,
}

impl DeadOptionFlattenItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-flatten:{}", self.value)
    }
}

pub fn dead_option_flatten(raw: &str) -> String {
    DeadOptionFlattenItem::new(raw).dead_method()
}
