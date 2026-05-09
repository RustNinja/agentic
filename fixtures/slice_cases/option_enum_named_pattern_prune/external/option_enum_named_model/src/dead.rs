pub struct DeadOptionEnumNamedItem {
    value: String,
}

impl DeadOptionEnumNamedItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-enum-named:{}", self.value)
    }
}

pub fn dead_option_enum_named(raw: &str) -> String {
    DeadOptionEnumNamedItem::new(raw).dead_method()
}
