pub struct DeadResultExpectItem {
    value: String,
}

impl DeadResultExpectItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-expect:{}", self.value)
    }
}

pub fn dead_result_expect(raw: &str) -> String {
    DeadResultExpectItem::new(raw).dead_method()
}
