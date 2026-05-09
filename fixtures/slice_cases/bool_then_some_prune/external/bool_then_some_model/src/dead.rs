pub struct DeadBoolThenSomeItem {
    value: String,
}

impl DeadBoolThenSomeItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-bool-then-some:{}", self.value)
    }
}

pub fn dead_bool_then_some(raw: &str) -> String {
    DeadBoolThenSomeItem::new(raw).dead_method()
}
