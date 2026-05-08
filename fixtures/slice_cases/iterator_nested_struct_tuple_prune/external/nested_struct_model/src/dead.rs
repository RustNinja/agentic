pub struct DeadNestedStructItem {
    value: String,
}

impl DeadNestedStructItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-nested-struct:{}", self.value)
    }
}

pub fn dead_nested_struct(raw: &str) -> String {
    DeadNestedStructItem::new(raw).render()
}
