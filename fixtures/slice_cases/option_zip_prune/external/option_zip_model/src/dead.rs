pub struct DeadOptionZipItem {
    value: String,
}

impl DeadOptionZipItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-zip:{}", self.value)
    }
}

pub fn dead_option_zip(raw: &str) -> String {
    DeadOptionZipItem::new(raw).dead_method()
}
