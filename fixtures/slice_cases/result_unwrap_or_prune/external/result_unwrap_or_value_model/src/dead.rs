pub struct DeadResultUnwrapOrValueItem {
    value: String,
}

impl DeadResultUnwrapOrValueItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-unwrap-or-value:{}", self.value)
    }
}

pub fn dead_result_unwrap_or_value(raw: &str) -> String {
    DeadResultUnwrapOrValueItem::new(raw).dead_method()
}
