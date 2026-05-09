pub struct DeadOptionStructItem {
    value: String,
}

impl DeadOptionStructItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-struct:{}", self.value)
    }
}

pub fn dead_option_struct(raw: &str) -> String {
    DeadOptionStructItem::new(raw).dead_method()
}
