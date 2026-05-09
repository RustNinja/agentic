pub struct DeadOptionExpectItem {
    value: String,
}

impl DeadOptionExpectItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-expect:{}", self.value)
    }
}

pub fn dead_option_expect(raw: &str) -> String {
    DeadOptionExpectItem::new(raw).dead_method()
}
