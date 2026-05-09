pub struct DeadOptionTupleStructItem {
    value: String,
}

impl DeadOptionTupleStructItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-tuple-struct:{}", self.value)
    }
}

pub fn dead_option_tuple_struct(raw: &str) -> String {
    DeadOptionTupleStructItem::new(raw).dead_method()
}
