pub struct DeadResultOptionItem {
    value: String,
}

impl DeadResultOptionItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-option:{}", self.value)
    }
}

pub fn dead_result_option(raw: &str) -> String {
    DeadResultOptionItem::new(raw).dead_method()
}
