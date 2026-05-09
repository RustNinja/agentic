pub struct DeadResultUnwrapDirectItem {
    value: String,
}

impl DeadResultUnwrapDirectItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-result-unwrap-direct:{}", self.value)
    }
}

pub fn dead_result_unwrap_direct(raw: &str) -> String {
    DeadResultUnwrapDirectItem::new(raw).dead_method()
}
