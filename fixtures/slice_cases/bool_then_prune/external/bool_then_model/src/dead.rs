pub struct DeadBoolThenItem {
    value: String,
}

impl DeadBoolThenItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-bool-then:{}", self.value)
    }
}

pub fn dead_bool_then(raw: &str) -> String {
    DeadBoolThenItem::new(raw).dead_method()
}
