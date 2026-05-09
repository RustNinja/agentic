pub struct DeadOptionTupleItem {
    value: String,
}

impl DeadOptionTupleItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-tuple:{}", self.value)
    }
}

pub fn dead_option_tuple(raw: &str) -> String {
    DeadOptionTupleItem::new(raw).dead_method()
}
